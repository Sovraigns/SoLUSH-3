// src/bin/gp_tester.rs

use anyhow::{anyhow, bail, Result};
use rand::{thread_rng, Rng};

// 1) We'll use the helper that reads creation code from a JSON artifact
use offchain::helpers::artifact::get_creation_code;

// 2) EvmRunner to deploy & run the interpreter
use offchain::runner::revm_runner::EvmRunner;

// 3) AST generation & mutation
use offchain::gp::generate::random_ast;
use offchain::gp::genetic_ops::mutated_ast;
use offchain::compiler::ast::UntypedAst;

fn main() -> Result<()> {
    // ----------------------------------------------------------------------
    // 1) Read the creation code (Push3Interpreter) using our helper function
    // ----------------------------------------------------------------------
    let creation_hex_filename = "../onchain/out/Push3Interpreter.sol/Push3Interpreter.json";
    let creation_bytes = get_creation_code(creation_hex_filename)?;
    println!(
        "Loaded creation code from {} ({} bytes)",
        creation_hex_filename,
        creation_bytes.len()
    );

    // 2) Create the ephemeral EVM (deploy the contract)
    let mut runner = EvmRunner::new(creation_bytes)?;
    println!("Deployed test interpreter at: 0x{:x}\n", runner.interpreter_addr);

    // 3) We'll generate random ASTs & mutate them
    let number_of_programs = 3; // how many random ASTs to try
    let max_depth = 4;         // max AST depth
    let mut rng = thread_rng();

    for i in 0..number_of_programs {
        // a) Generate a random AST
        let ast: UntypedAst = random_ast(&mut rng, 0, max_depth);

        println!("=== Program {} ===", i);
        println!("Random AST:\n{:#?}", ast);

        // b) Run the original AST
        match runner.run_ast(&ast) {
            Ok(outputs) => {
                println!("Original AST ran successfully!");
                println!("Final CODE stack: {:?}", outputs.final_code_stack);
                println!("Final EXEC stack: {:?}", outputs.final_exec_stack);
                println!("Final INT stack:  {:?}", outputs.final_int_stack);

                // c) Mutate it a few times
                let mutated1 = mutated_ast(&ast, &mut rng, 0, max_depth);
                println!("\n-- MUTATED 1 --\n{:#?}", mutated1);

                match runner.run_ast(&mutated1) {
                    Ok(m1_out) => {
                        println!("MUTATED 1 ran successfully!");
                        println!("Final INT stack: {:?}", m1_out.final_int_stack);
                    }
                    Err(e) => {
                        println!("MUTATED 1 error: {}", e);
                    }
                }

                let mutated2 = mutated_ast(&ast, &mut rng, 0, max_depth);
                println!("\n-- MUTATED 2 --\n{:#?}", mutated2);

                match runner.run_ast(&mutated2) {
                    Ok(m2_out) => {
                        println!("MUTATED 2 ran successfully!");
                        println!("Final INT stack: {:?}", m2_out.final_int_stack);
                    }
                    Err(e) => {
                        println!("MUTATED 2 error: {}", e);
                    }
                }
            }
            Err(e) => {
                // If the original fails, we won't bother mutating it
                println!("Original AST error: {}", e);
            }
        }

        println!("=== End of Program {} ===\n", i);
    }

    println!("Finished running & mutating {} random programs.", number_of_programs);
    Ok(())
}
