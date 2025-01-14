// src/bin/symreg_experiment.rs

use anyhow::Result;
use rand::{thread_rng, Rng};
use offchain::compiler::ast::{UntypedAst, Push3Ast};
use offchain::helpers::artifact::get_creation_code;
use offchain::runner::revm_runner::{
    EvmRunner, 
    Push3InterpreterInputs, 
    Push3InterpreterOutputs
};
use offchain::gp::generate_spec::{ranmdom_code_fixed};
use offchain::gp::mutation::mutate_by_index;
use offchain::compiler::push3_describtor::make_sublist_descriptor;
use ethers::types::U256;
use ethers::abi::{encode, Token};

// 1) Generate (x, y) samples for f(x) = 3x^2 + x + 3
fn generate_samples() -> Vec<(i32, i32)> {
    let mut samples = Vec::new();
    for x in -5..=5 {
        let y = 3 * x * x + x + 3;
        samples.push((x, y));
    }
    samples
}

// 2) Evaluate a single AST on a single input `x`, returning the integer
//    that ends up on top of the final int stack (or 0 if empty).
fn evaluate_ast_on_x(
    runner: &mut EvmRunner,
    ast: &UntypedAst,
    x: i32,
) -> i32 {
    // a) Convert AST => push3 bytecode
    let code_bytes = ast.to_bytecode();
    let code_len = code_bytes.len() as u32;

    // b) Build a sublist descriptor
    let descriptor = make_sublist_descriptor(0, code_len);

    // c) Provide x in `init_int_stack`
    let inputs = Push3InterpreterInputs {
        code: code_bytes,
        init_code_stack: Vec::new(),
        init_exec_stack: vec![descriptor],
        init_int_stack: vec![x as i128],
    };

    // d) Actually run the interpreter
    let outputs = match runner.run_interpreter(&inputs) {
        Ok(o) => o,
        Err(_) => {
            // If the call fails or reverts, return 0
            return 0;
        }
    };

    // e) If final_int_stack is empty => 0, else top item
    if outputs.final_int_stack.is_empty() {
        0
    } else {
        *outputs.final_int_stack.last().unwrap() as i32
    }
}

// 3) Evaluate an AST on all (x, y) samples => compute MSE
fn evaluate_fitness(
    runner: &mut EvmRunner,
    ast: &UntypedAst,
    samples: &[(i32, i32)]
) -> f64 {
    let mut error_sum = 0.0;
    for &(x, target_y) in samples {
        let predicted = evaluate_ast_on_x(runner, ast, x);
        let diff = (predicted - target_y) as f64;
        error_sum += diff * diff;
    }
    error_sum / samples.len() as f64
}

fn main() -> Result<()> {
    // A) Generate samples for f(x)=3x^2 + x + 3
    let samples = generate_samples();

    // B) Create an ephemeral EVM runner (Push3Interpreter)
    let creation_hex_filename = "../onchain/out/Push3Interpreter.sol/Push3Interpreter.json";
    let creation_bytes = get_creation_code(creation_hex_filename)?;
    let mut runner = EvmRunner::new(creation_bytes)?;

    // C) Setup GP parameters
    let pop_size = 1000;       // population size
    let max_points = 15;      // max "size" for random code
    let generations = 100;    // how many generations we run
    let early_stop_threshold = 1.0; // if error < this => we can stop

    let mut rng = thread_rng();

    // D) Initialize random population
    let mut population: Vec<UntypedAst> = (0..pop_size)
        .map(|_| ranmdom_code_fixed(&mut rng, max_points))
        .collect();

    // E) Main GP Loop
    for gen in 0..generations {
        // 1) Evaluate each => store (ast, error)
        let mut scored: Vec<(UntypedAst, f64)> = population.into_iter()
            .map(|ast| {
                let err = evaluate_fitness(&mut runner, &ast, &samples);
                (ast, err)
            })
            .collect();

        // 2) Sort ascending by error
        scored.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        // 3) Print best error
        let best_err = scored[0].1;
        println!("Generation {gen}, best error = {best_err}");

        // 4) Check if best is below threshold => early stop
        if best_err < early_stop_threshold {
            println!("Best error < {early_stop_threshold}, stopping early!");
            population = scored.into_iter().map(|(a, _)| a).collect();
            break;
        }

        // 5) Keep best half
        let half = scored.len() / 2;
        let survivors = &scored[..half];

        // 6) Refill population: half mutated, half fresh random
        let mut new_pop = Vec::with_capacity(scored.len());
        // a) keep survivors
        for &(ref ast, _) in survivors {
            new_pop.push(ast.clone());
        }

        // b) how many to add?
        let remainder_size = pop_size - new_pop.len();
        // we do half mutated, half new random
        let mutated_count = remainder_size / 2;
        let random_count = remainder_size - mutated_count;

        // c) add mutated
        for _ in 0..mutated_count {
            let idx = rng.gen_range(0..half);
            let mutated = mutate_by_index(&survivors[idx].0, &mut rng, max_points);
            new_pop.push(mutated);
        }

        // d) add brand-new random
        for _ in 0..random_count {
            let fresh = ranmdom_code_fixed(&mut rng, max_points);
            new_pop.push(fresh);
        }

        // store for next generation
        population = new_pop;
    }

    // F) Final Print
    println!("\n=== Final Population ===");
    for (i, ast) in population.iter().enumerate() {
        let err = evaluate_fitness(&mut runner, ast, &samples);
        println!("Subject #{i}, err={err}, ast={:?}", ast);
    }

    Ok(())
}
