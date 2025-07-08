// src/bin/symreg_experiment.rs

use anyhow::Result;
use rand::{thread_rng, Rng};

// Our GP + compiler modules (adjust paths as needed)
use offchain::compiler::ast::{UntypedAst, Push3Ast};
use offchain::helpers::artifact::get_creation_code;
use offchain::runner::revm_runner::{
    EvmRunner, 
    Push3InterpreterInputs,
};
use offchain::compiler::push3_describtor::make_sublist_descriptor;

// Our GP operators
use offchain::gp::generate_spec::ranmdom_code_fixed;
use offchain::gp::mutation::{mutate_by_index, crossover_by_index};

// Ethers + ABI
use ethers::types::U256;
use ethers::abi::{encode, Token};

/// 1) Generate (x, y) samples for f(x)=3x^2 + x + 3
fn generate_samples() -> Vec<(i32, i32)> {
    let mut samples = Vec::new();
    for x in -5..=5 {
        let y = 3 * x * x + x + 3;
        samples.push((x, y));
    }
    samples
}

/// 2) Evaluate a single AST on a single input `x` by 
/// pushing `x` onto the int stack, running in EVM, 
/// and reading the final top of the int stack.
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
        init_bool_stack: Vec::new(),
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
        i32::MAX
    } else {
        *outputs.final_int_stack.last().unwrap() as i32
    }
}

/// 3) Evaluate an AST on all (x, y) samples => compute gradual fitness
fn evaluate_fitness(
    runner: &mut EvmRunner,
    ast: &UntypedAst,
    samples: &[(i32, i32)]
) -> f64 {
    let mut total_fitness = 0.0;
    let mut successful_evaluations = 0;
    
    for &(x, target_y) in samples {
        let predicted = evaluate_ast_on_x(runner, ast, x);
        
        // Handle execution failures gracefully
        if predicted == i32::MAX {
            // Program failed to execute - give small partial credit
            total_fitness += 0.1;
        } else {
            successful_evaluations += 1;
            let diff = (predicted - target_y).abs() as f64;
            
            // Gradual fitness: reward getting closer to target
            // Use inverse exponential decay for smoother gradients
            let sample_fitness = if diff == 0.0 {
                1000.0  // Perfect match gets high reward
            } else if diff <= 1.0 {
                100.0 / (1.0 + diff)  // Very close gets good reward
            } else if diff <= 10.0 {
                50.0 / (1.0 + diff * 0.5)  // Moderately close gets decent reward  
            } else if diff <= 100.0 {
                20.0 / (1.0 + diff * 0.1)  // Within range gets some reward
            } else {
                10.0 / (1.0 + diff * 0.01)  // Far but finite gets minimal reward
            };
            
            total_fitness += sample_fitness;
        }
    }
    
    // Bonus for programs that execute successfully on all samples
    if successful_evaluations == samples.len() {
        total_fitness *= 1.2;  // 20% bonus for reliability
    }
    
    // Return average fitness per sample (higher = better)
    total_fitness / samples.len() as f64
}


fn main() -> Result<()> {
    // 1) Generate samples for f(x)=3x^2 + x + 3
    let samples = generate_samples();

    // 2) Create ephemeral EVM runner for the interpreter
    let creation_hex_filename = "../onchain/out/Push3Interpreter.sol/Push3Interpreter.json";
    let creation_bytes = get_creation_code(creation_hex_filename)?;
    let mut runner = EvmRunner::new(creation_bytes)?;

    // 3) GP parameters
    let pop_size = 100;  // must be divisible by 4 for our scheme
    let generations = 5;
    let early_stop_threshold = 1.0;
    let max_points = 10;

    assert_eq!(pop_size % 4, 0, "pop_size must be multiple of 4 for our scheme");
    let quarter = pop_size / 4;

    let mut rng = thread_rng();

    // 4) Initialize population (random)
    let mut population: Vec<UntypedAst> = (0..pop_size)
        .map(|_| ranmdom_code_fixed(&mut rng, max_points))
        .collect();

    // 5) Main GP loop
    for gen in 0..generations {
        // (a) Evaluate
        let mut scored: Vec<(UntypedAst, f64)> = population.into_iter()
            .map(|ast| {
                let fitness = evaluate_fitness(&mut runner, &ast, &samples);
                (ast, fitness)
            })
            .collect();

        // (b) Sort descending by fitness (higher = better)
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // (c) Print generation summary
        let best_fitness = scored[0].1;
        println!("\n=== Generation {gen} ===");
        println!("Best fitness = {best_fitness}");

        // Print top 5 subjects in pretty format
        // let top_n = 5.min(scored.len());
        // println!("Top {top_n} subjects:");
        // for i in 0..top_n {
        //     println!(
        //         "  #{i}, err={}, AST:\n{:#?}",
        //         scored[i].1,
        //         scored[i].0
        //     );
        // }

        // early stop? (high fitness = good)
        if best_fitness > 900.0 {  // Close to perfect score of 1000+
            println!("Best fitness > 900.0, stopping early!");
            // we can keep 'scored' for final if we want
            population = scored.into_iter().map(|(a, _)| a).collect();
            break;
        }

        // (d) Reproduction => build new population
        let mut new_pop = Vec::with_capacity(pop_size);

        // keep best quarter
        for i in 0..quarter {
            new_pop.push(scored[i].0.clone());
        }

        // add random quarter
        for _ in 0..quarter {
            let fresh = ranmdom_code_fixed(&mut rng, max_points);
            new_pop.push(fresh);
        }

        // crossover quarter
        let top_half_count = scored.len() / 2;
        let crossover_pairs = quarter / 2;  // each yields 2 kids => quarter total
        for _ in 0..crossover_pairs {
            let idx1 = rng.gen_range(0..top_half_count);
            let idx2 = rng.gen_range(0..top_half_count);
            let (child1, child2) = crossover_by_index(
                &scored[idx1].0,
                &scored[idx2].0,
                &mut rng
            );
            new_pop.push(child1);
            new_pop.push(child2);
        }

        // mutate quarter
        while new_pop.len() < pop_size {
            let idx = rng.gen_range(0..top_half_count);
            let mutated = mutate_by_index(&scored[idx].0, &mut rng, max_points);
            new_pop.push(mutated);
        }

        population = new_pop;
    }

    // 6) Final population => evaluate & sort, then print only top 10
    println!("\n=== Final Population (Top 10) ===");

    // a) Evaluate each item in the final population
    let mut final_scored: Vec<(UntypedAst, f64)> = population
        .into_iter()
        .map(|ast| {
            let fitness = evaluate_fitness(&mut runner, &ast, &samples);
            (ast, fitness)
        })
        .collect();
    
    // b) Sort descending by fitness (higher = better)
    final_scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    // c) Print the top 10 (or fewer if the population is smaller)
    let top_n = 10.min(final_scored.len());
    for i in 0..top_n {
        let (ref ast, fitness) = final_scored[i];
        println!(
            "Subject #{i}, fitness={fitness}, AST:\n{:#?}",
            ast
        );
    }
    
    Ok(())
}
