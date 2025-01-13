// src/main.rs

mod compiler;

use std::env;
use compiler::ast::{
    parse_string_to_sexpr,
    sexpr_to_untyped,
    Push3Ast, // The trait
    UntypedAst, // The untyped AST enum
};

use anyhow::{anyhow, bail, Result};
use hex;
use database::CacheDB;
use revm::{
    context::Context,
    context_interface::result::{ExecutionResult, Output},
    database_interface::EmptyDB,
    handler::EthHandler,
    primitives::{Bytes, TxKind},
    EvmCommit, MainEvm,
};
use ethers::abi::{encode, Token};
use ethers::abi::{ParamType, decode};
use ethers::types::U256;
use serde::Deserialize; // <--- NEW: for deserializing from JSON

#[derive(Debug, Deserialize)]
struct BytecodeObject {
    object: String,
}

#[derive(Debug, Deserialize)]
struct MyContractArtifact {
    bytecode: BytecodeObject,
}

fn main() -> Result<()> {
    // 1) Read CLI arguments
    let args: Vec<String> = std::env::args().collect();
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

    // 4) Convert UntypedAst => Push3 bytecode
    let bytecode = ast.to_bytecode();

    // 5) Print the resulting AST and bytecode (in hex)
    println!("AST: {:?}", ast);
    println!("Bytecode (hex): 0x{}", hex::encode(&bytecode));

    // ----------------------------------------------------------------------
    // NEW: Read the contract creation code (bytecode) from a JSON artifact
    // ----------------------------------------------------------------------
    // Example JSON (Forge artifact) might look like:
    // {
    //   "bytecode": "60808060...0033"   <--- entire creation code
    //   ... other fields ...
    // }

    // Path to your JSON artifact. Adjust as needed:
    let creation_hex_filename = "../onchain/out/Push3Interpreter.sol/Push3Interpreter.json";

    // Read and parse the JSON artifact
    let creation_json = std::fs::read_to_string(creation_hex_filename)
        .map_err(|e| anyhow!("Failed to read JSON file {}: {}", creation_hex_filename, e))?;
    let contract_artifact: MyContractArtifact = serde_json::from_str(&creation_json)
        .map_err(|e| anyhow!("Failed to parse JSON artifact: {}", e))?;
    
    // Extract the "bytecode" field from the JSON
    let raw_hex = &contract_artifact.bytecode.object;
    let stripped_hex = raw_hex.trim_start_matches("0x");

    // ----------------------------------------------------------------------
    // The rest of the deployment+call logic remains mostly unchanged
    // ----------------------------------------------------------------------

    // 2) Convert hex string to raw bytes.
    let creation_bytes = match hex::decode(stripped_hex) {
        Ok(bytes) => bytes,
        Err(e) => bail!("Invalid hex for creation code: {e}"),
    };

    // 3) Build a new EVM instance with a "Create" transaction:
    let mut evm = MainEvm::new(
        // a) Prepare context with a chained transaction
        Context::builder()
            .modify_tx_chained(|tx| {
                tx.transact_to = TxKind::Create;
                tx.data = Bytes::from(creation_bytes.clone());
            })
            // b) Provide an in-memory DB for ephemeral state
            .with_db(CacheDB::<EmptyDB>::default()),
        // c) The "EthHandler" for block/tx environment logic
        EthHandler::default(),
    );

    println!("Deploying contract with creation code size: {} bytes", creation_bytes.len());

    // 4) Execute the CREATE transaction
    let creation_result = evm.exec_commit()?;
    // => an ExecutionResult

    // 5) Check success
    let ExecutionResult::Success {
        output: Output::Create(_, Some(deployed_addr)),
        ..
    } = creation_result
    else {
        bail!("Deployment failed or didn't return an address: {creation_result:#?}");
    };

    println!("Deployed contract at: 0x{:x}", deployed_addr);

    // 1) Prepare the function selector for runInterpreter
    let func_selector = &ethers::utils::id("runInterpreter(bytes,uint256[],uint256[],int256[])")[0..4];

    // 2) Convert your compiled “Push3” bytecode into the `bytes` argument
    let code_arg = Token::Bytes(bytecode.clone());

    // 3) Build a “SUBLIST descriptor” for the entire bytecode.
    let tag_bits    = U256::from(3u64)                  << 248; // CodeTag.SUBLIST = 3
    let offset_bits = U256::from(0u64)                  << 216; // offset=0
    let length_bits = U256::from(bytecode.len() as u64) << 184; // length = bytecode.len()
    let low_184     = U256::zero();

    let sublist_descriptor = tag_bits | offset_bits | length_bits | low_184;

    // Now we put that descriptor as the one and only item in `initExecStack`.
    let init_exec_stack = Token::Array(vec![
        Token::Uint(sublist_descriptor)
    ]);

    // For the other two arrays (codeStack, intStack), we’ll just leave them empty:
    let empty_uint256_array = Token::Array(vec![]); // codeStack
    let empty_int256_array  = Token::Array(vec![]); // intStack

    // 4) Encode all arguments in the correct order:
    let encoded_args = encode(&[
        code_arg,
        empty_uint256_array.clone(),   // initCodeStack
        init_exec_stack,               // initExecStack
        empty_uint256_array.clone(),   // initIntStack
    ]);

    // 5) Concatenate selector + encoded arguments to form the full call data
    let mut call_data = Vec::from(func_selector);
    call_data.extend_from_slice(&encoded_args);

    // 6) Modify the transaction to call `runInterpreter` with the encoded data
    evm.context.modify_tx(|tx| {
        tx.transact_to = TxKind::Call(deployed_addr);
        tx.data = Bytes::from(call_data);
        tx.nonce += 1; // increment nonce to avoid reuse
    });

    // 7) Execute the call.
    let call_result = evm.transact()?;
    match &call_result.result {
        ExecutionResult::Success {
            output: Output::Call(return_data),
            ..
        } => {
            println!("Call succeeded, returned data: 0x{}", hex::encode(return_data));

            // Decode (uint256[], uint256[], int256[])
            let param_types = &[
                ParamType::Array(Box::new(ParamType::Uint(256))), // finalCodeStack
                ParamType::Array(Box::new(ParamType::Uint(256))), // finalExecStack
                ParamType::Array(Box::new(ParamType::Int(256))),  // finalIntStack
            ];
            
            let decoded = decode(param_types, return_data)
                .map_err(|e| anyhow!("Failed to decode return data: {e}"))?;
            
            println!("Decoded return: {:?}", decoded);
        }
        ExecutionResult::Revert { gas_used, output } => {
            bail!("Call reverted: gas used = {gas_used:?}, output = {:?}", output);
        }
        other => {
            bail!("Call failed: {other:?}");
        }
    }
    Ok(())
}
