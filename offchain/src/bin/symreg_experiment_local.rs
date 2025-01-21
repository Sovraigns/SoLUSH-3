// src/bin/symreg_experiment_local.rs

use anyhow::Result;
use rand::{thread_rng, Rng};

// Our GP + compiler modules (adjust paths if needed)
use offchain::compiler::ast::{UntypedAst, Push3Ast};
use offchain::helpers::artifact::get_creation_code;
use offchain::runner::revm_runner::{EvmRunner, Push3InterpreterInputs};
use offchain::compiler::push3_describtor::make_sublist_descriptor;

// Our random code and local mutation
use offchain::gp::generate_spec::ranmdom_code_fixed;
use offchain::gp::local_mutation::local_mutation_fixed;

// Ethers + ABI
use ethers::types::U256;
use ethers::abi::{encode, Token};

/// 1) Generate (x, y) samples for f(x) = 3x^2 + x + 3
fn generate_samples() -> Vec<(i32, i32)> {
    let mut samples = Vec::new();
    for x in -5..=5 {
        let y = 3 * x * x + x + 3;
        samples.push((x, y));
    }
    samples
}

/// 2) Evaluate a single AST on a single input `x`, returning the final top of the int stack.
fn evaluate_ast_on_x(
    runner: &mut EvmRunner,
    ast: &UntypedAst,
    x: i32,
) -> i32 {
    // Convert AST => push3 bytecode
    let code_bytes = ast.to_bytecode();
    let code_len = code_bytes.len() as u32;

    // Build a sublist descriptor
    let descriptor = make_sublist_descriptor(0, code_len);

    // Provide x in `init_int_stack`
    let inputs = Push3InterpreterInputs {
        code: code_bytes,
        init_code_stack: Vec::new(),
        init_exec_stack: vec![descriptor],
        init_int_stack: vec![x as i128],
    };

    // Run
    let outputs = match runner.run_interpreter(&inputs) {
        Ok(o) => o,
        Err(_) => {
            // If revert => 0
            return 0;
        }
    };

    if outputs.final_int_stack.is_empty() {
        i32::MAX  // or 0, or i32::MAX to strongly penalize weird runs
    } else {
        *outputs.final_int_stack.last().unwrap() as i32
    }
}

/// 3) Evaluate an AST on all samples => compute MSE
fn evaluate_fitness(
    runner: &mut EvmRunner,
    ast: &UntypedAst,
    samples: &[(i32, i32)]
) -> f64 {
    let mut error_sum = 0.0;
    for &(x, target_y32) in samples {
        // cast to i64
        let target_y = target_y32 as i64;
        
        // evaluate in i64
        let predicted_i64 = evaluate_ast_on_x(runner, ast, x) as i64;
        
        // wrapping_sub to avoid panic
        let diff = predicted_i64.wrapping_sub(target_y);
        error_sum += (diff as f64) * (diff as f64);
    }
    error_sum / samples.len() as f64
}

fn main() -> Result<()> {
    // 1) Generate samples
    let samples = generate_samples();

    // 2) Setup ephemeral EVM runner
    let creation_hex_filename = "../onchain/out/Push3Interpreter.sol/Push3Interpreter.json";
    let creation_bytes = get_creation_code(creation_hex_filename)?;
    let mut runner = EvmRunner::new(creation_bytes)?;

    // 3) GP parameters
    let pop_size = 3600;  // must be multiple of 3 for our scheme
    let generations = 100;
    let early_stop_threshold = 1.0;
    let max_points = 12;  // random code size limit

    // Ensure pop_size is multiple of 3
    assert_eq!(pop_size % 3, 0, "pop_size must be multiple of 3 for our scheme");
    let third = pop_size / 3;

    let mut rng = thread_rng();

    // 4) Initialize random population
    let mut population: Vec<UntypedAst> = (0..pop_size)
        .map(|_| ranmdom_code_fixed(&mut rng, max_points))
        .collect();

    // 5) Main loop
    for gen in 0..generations {
        // a) Evaluate
        let mut scored: Vec<(UntypedAst, f64)> = population.into_iter()
            .map(|ast| {
                let err = evaluate_fitness(&mut runner, &ast, &samples);
                (ast, err)
            })
            .collect();
        
        // b) Sort ascending by error
        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let best_err = scored[0].1;
        println!("\n=== Generation {gen} ===");
        println!("Best error = {best_err}");

        // (Optionally) Print top 5 in pretty format
        let top_n = 5.min(scored.len());
        println!("Top {top_n} subjects:");
        for i in 0..top_n {
            println!(
                "  #{i}, err={}, AST:\n{:#?}",
                scored[i].1, 
                scored[i].0
            );
        }

        // check early stop
        if best_err < early_stop_threshold {
            println!("Best error < {early_stop_threshold}, stopping early!");
            population = scored.into_iter().map(|(a, _)| a).collect();
            break;
        }

        // c) Build new population:
        //   - 1/3 = best survivors
        //   - 1/3 = fresh random
        //   - 1/3 = local mutation
        let mut new_pop = Vec::with_capacity(pop_size);

        // keep best third
        for i in 0..third {
            new_pop.push(scored[i].0.clone());
        }

        // random third
        for _ in 0..third {
            let fresh = ranmdom_code_fixed(&mut rng, max_points);
            new_pop.push(fresh);
        }

        // local mutation third
        let top_half_count = scored.len() / 2; 
        // we'll pick parents from top half for local mutation
        while new_pop.len() < pop_size {
            let idx = rng.gen_range(0..top_half_count);
            // we assume you have local_mutation_fixed
            // if you prefer passing a custom InstructionSet, do local_mutation(..., &instr_set)
            let mutated = offchain::gp::local_mutation::local_mutation_fixed(
                &scored[idx].0,
                &mut rng
            );
            new_pop.push(mutated);
        }

        // store for next generation
        population = new_pop;
    }

    // 6) Final => evaluate & sort => top 10
    println!("\n=== Final Population (Top 10) ===");
    let mut final_scored: Vec<(UntypedAst, f64)> = population
        .into_iter()
        .map(|ast| {
            let err = evaluate_fitness(&mut runner, &ast, &samples);
            (ast, err)
        })
        .collect();
    final_scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let top_n = 10.min(final_scored.len());
    for i in 0..top_n {
        let (ref ast, err) = final_scored[i];
        println!("Subject #{i}, err={err}, AST:\n{:#?}", ast);
    }

    Ok(())
}
