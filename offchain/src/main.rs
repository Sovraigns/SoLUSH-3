// src/main.rs

mod compiler;
mod runner;

use std::env;

// We’ll use your existing AST definitions for parsing
use compiler::ast::{
    parse_string_to_sexpr,
    sexpr_to_untyped,
    Push3Ast, // The trait
    UntypedAst, // The untyped AST enum
};

use runner::revm_runner::{EvmRunner, Push3InterpreterOutputs}; // <--- We import our EvmRunner

use anyhow::{anyhow, bail, Result};
use hex;
use serde::Deserialize; // For JSON artifact loading

// If you still need these (not strictly required anymore):
// use database::CacheDB;
// use revm::{
//     context::Context,
//     context_interface::result::{ExecutionResult, Output},
//     database_interface::EmptyDB,
//     handler::EthHandler,
//     primitives::{Bytes, TxKind},
//     EvmCommit, MainEvm,
// };
use ethers::abi::{decode, ParamType};
use ethers::types::U256;

// ----------------------------------------------------------------------
// JSON artifact struct, as before
// ----------------------------------------------------------------------
#[derive(Debug, Deserialize)]
struct BytecodeObject {
    object: String,
}

#[derive(Debug, Deserialize)]
struct MyContractArtifact {
    bytecode: BytecodeObject,
}

// ----------------------------------------------------------------------
// MAIN
// ----------------------------------------------------------------------
fn main() -> Result<()> {
    // 1) Read CLI arguments for the Push3 program
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: cargo run -- '<program>'");
        eprintln!("Example: cargo run -- '((3 5 +) DUP MUL)'");
        std::process::exit(1);
    }
    let program_str = &args[1];

    // 2) Parse the string into an S-expression
    let sexpr = parse_string_to_sexpr(program_str)
        .map_err(|e| anyhow!("Error parsing S-expression: {}", e))?;

    // 3) Convert SExpr => UntypedAst
    let ast: UntypedAst = sexpr_to_untyped(&sexpr)
        .map_err(|e| anyhow!("Error converting to UntypedAst: {}", e))?;

    // 4) Convert UntypedAst => Push3 bytecode (for debugging, so we can see it)
    let bytecode = ast.to_bytecode();

    // 5) Print the resulting AST and bytecode (in hex)
    println!("AST: {:?}", ast);
    println!("Bytecode (hex): 0x{}", hex::encode(&bytecode));

    // ----------------------------------------------------------------------
    // Load the creation code (bytecode) from a JSON artifact
    // ----------------------------------------------------------------------
    let creation_hex_filename = "../onchain/out/Push3Interpreter.sol/Push3Interpreter.json";
    let creation_json = std::fs::read_to_string(creation_hex_filename)
        .map_err(|e| anyhow!("Failed to read JSON file {}: {}", creation_hex_filename, e))?;
    let contract_artifact: MyContractArtifact = serde_json::from_str(&creation_json)
        .map_err(|e| anyhow!("Failed to parse JSON artifact: {}", e))?;

    let raw_hex = &contract_artifact.bytecode.object;
    let stripped_hex = raw_hex.trim_start_matches("0x");

    let creation_bytes = match hex::decode(stripped_hex) {
        Ok(bytes) => bytes,
        Err(e) => bail!("Invalid hex for creation code: {e}"),
    };

    println!(
        "Loaded creation code ({} bytes) from {}",
        creation_bytes.len(),
        creation_hex_filename
    );

    // ----------------------------------------------------------------------
    // 6) Deploy the interpreter in an ephemeral EVM using `EvmRunner`
    // ----------------------------------------------------------------------
    let mut runner = EvmRunner::new(creation_bytes)?;
    println!("Interpreter deployed successfully in ephemeral EVM at: 0x{:x}", runner.interpreter_addr);

    // ----------------------------------------------------------------------
    // 7) Evaluate the AST in our EVM
    // ----------------------------------------------------------------------
    // We'll call `runner.run_ast(&ast)`, which:
    // - Converts AST => push3 code,
    // - Builds a sublist descriptor in the exec stack,
    // - Calls `runInterpreter`,
    // - Returns final stacks in a `Push3InterpreterOutputs`.
    let outputs = runner.run_ast(&ast)?;
    println!("Ran the AST successfully!");

    // 8) Print the final stacks
    println!("Final CODE stack: {:?}", outputs.final_code_stack);
    println!("Final EXEC stack: {:?}", outputs.final_exec_stack);
    println!("Final INT stack: {:?}", outputs.final_int_stack);

    Ok(())
}
