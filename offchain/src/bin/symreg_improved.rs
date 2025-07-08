// src/bin/symreg_improved.rs
// Enhanced symbolic regression with improved genetic operators

use anyhow::Result;
use rand::{thread_rng, Rng};

// Our GP + compiler modules 
use offchain::compiler::ast::{UntypedAst, Push3Ast};
use offchain::helpers::artifact::get_creation_code;
use offchain::runner::revm_runner::{
    EvmRunner, 
    Push3InterpreterInputs,
};
use offchain::compiler::push3_describtor::make_sublist_descriptor;

// Our enhanced GP operators
use offchain::gp::generate_spec::ranmdom_code_fixed;
use offchain::gp::mutation::{
    mutate_by_index, crossover_by_index, point_mutate, 
    size_aware_crossover, size_limited_mutate, get_subtree_size
};

/// 1) Generate (x, y) samples for f(x)=3x^2 + x + 3
fn generate_samples() -> Vec<(i32, i32)> {
    let mut samples = Vec::new();
    for x in -5..=5 {
        let y = 3 * x * x + x + 3;
        samples.push((x, y));
    }
    samples
}

/// 2) Evaluate a single AST on a single input `x`
fn evaluate_ast_on_x(
    runner: &mut EvmRunner,
    ast: &UntypedAst,
    x: i32,
) -> i32 {
    let code_bytes = ast.to_bytecode();
    let code_len = code_bytes.len() as u32;
    let descriptor = make_sublist_descriptor(0, code_len);

    let inputs = Push3InterpreterInputs {
        code: code_bytes,
        init_code_stack: Vec::new(),
        init_exec_stack: vec![descriptor],
        init_int_stack: vec![x as i128],
        init_bool_stack: Vec::new(),
    };

    let outputs = match runner.run_interpreter(&inputs) {
        Ok(o) => o,
        Err(_) => return i32::MAX,
    };

    if outputs.final_int_stack.is_empty() {
        i32::MAX
    } else {
        *outputs.final_int_stack.last().unwrap() as i32
    }
}

/// 3) Enhanced fitness function with gradual rewards
fn evaluate_fitness(
    runner: &mut EvmRunner,
    ast: &UntypedAst,
    samples: &[(i32, i32)]
) -> f64 {
    let mut total_fitness = 0.0;
    let mut successful_evaluations = 0;
    
    for &(x, target_y) in samples {
        let predicted = evaluate_ast_on_x(runner, ast, x);
        
        if predicted == i32::MAX {
            total_fitness += 0.1;
        } else {
            successful_evaluations += 1;
            let diff = (predicted - target_y).abs() as f64;
            
            let sample_fitness = if diff == 0.0 {
                1000.0
            } else if diff <= 1.0 {
                100.0 / (1.0 + diff)
            } else if diff <= 10.0 {
                50.0 / (1.0 + diff * 0.5)
            } else if diff <= 100.0 {
                20.0 / (1.0 + diff * 0.1)
            } else {
                10.0 / (1.0 + diff * 0.01)
            };
            
            total_fitness += sample_fitness;
        }
    }
    
    // Bonus for reliability
    if successful_evaluations == samples.len() {
        total_fitness *= 1.2;
    }
    
    // Parsimony pressure: slight penalty for very large programs
    let size_penalty = match get_subtree_size(ast) {
        s if s > 30 => 0.9,  // 10% penalty for very large programs
        s if s > 20 => 0.95, // 5% penalty for large programs  
        _ => 1.0,            // No penalty for reasonable sizes
    };
    
    (total_fitness / samples.len() as f64) * size_penalty
}

fn main() -> Result<()> {
    println!("=== Enhanced Symbolic Regression Experiment ===");
    
    // 1) Generate samples
    let samples = generate_samples();
    println!("Target function: f(x) = 3x² + x + 3");
    println!("Test samples: {:?}", samples);

    // 2) Create EVM runner
    let creation_hex_filename = "../onchain/out/Push3Interpreter.sol/Push3Interpreter.json";
    let creation_bytes = get_creation_code(creation_hex_filename)?;
    let mut runner = EvmRunner::new(creation_bytes)?;

    // 3) Enhanced GP parameters
    let pop_size = 200;  // Reasonable size for testing
    let generations = 30;
    let max_points = 15;
    let max_size = 25;   // Size limit to prevent bloat

    assert_eq!(pop_size % 4, 0, "pop_size must be multiple of 4");
    let quarter = pop_size / 4;

    let mut rng = thread_rng();

    // 4) Initialize population
    let mut population: Vec<UntypedAst> = (0..pop_size)
        .map(|_| ranmdom_code_fixed(&mut rng, max_points))
        .collect();

    println!("\nStarting evolution with enhanced genetic operators:");
    println!("- Point mutation (fine-tuned changes)");
    println!("- Size-aware crossover (balanced exchanges)");  
    println!("- Size-limited mutation (bloat prevention)");
    println!("- Parsimony pressure (size penalties)");

    // 5) Enhanced GP loop
    for gen in 0..generations {
        // (a) Evaluate with enhanced fitness
        let mut scored: Vec<(UntypedAst, f64)> = population.into_iter()
            .map(|ast| {
                let fitness = evaluate_fitness(&mut runner, &ast, &samples);
                (ast, fitness)
            })
            .collect();

        // (b) Sort descending by fitness
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // (c) Print generation summary with size info
        let best_fitness = scored[0].1;
        let avg_fitness: f64 = scored.iter().map(|(_, f)| f).sum::<f64>() / scored.len() as f64;
        let avg_size: f64 = scored.iter().map(|(ast, _)| get_subtree_size(ast) as f64).sum::<f64>() / scored.len() as f64;
        let best_size = get_subtree_size(&scored[0].0);
        
        println!("\n=== Generation {gen} ===");
        println!("Best fitness: {:.2} (size: {})", best_fitness, best_size);
        println!("Avg fitness: {:.2}, Avg size: {:.1}", avg_fitness, avg_size);

        // Early stop for excellent solutions
        if best_fitness > 800.0 {
            println!("Excellent solution found! Stopping early.");
            population = scored.into_iter().map(|(a, _)| a).collect();
            break;
        }

        // (d) Enhanced reproduction strategy
        let mut new_pop = Vec::with_capacity(pop_size);

        // Keep elite quarter (best performers)
        for i in 0..quarter {
            new_pop.push(scored[i].0.clone());
        }

        // Add diverse quarter with size-limited mutation
        for _ in 0..quarter {
            let idx = rng.gen_range(0..scored.len() / 2);
            let mutated = size_limited_mutate(&scored[idx].0, &mut rng, max_points, max_size);
            new_pop.push(mutated);
        }

        // Enhanced crossover quarter (size-aware)
        let top_half_count = scored.len() / 2;
        let crossover_pairs = quarter / 2;
        for _ in 0..crossover_pairs {
            let idx1 = rng.gen_range(0..top_half_count);
            let idx2 = rng.gen_range(0..top_half_count);
            
            // Mix of regular and size-aware crossover
            let (child1, child2) = if rng.gen::<f64>() < 0.7 {
                size_aware_crossover(&scored[idx1].0, &scored[idx2].0, &mut rng)
            } else {
                crossover_by_index(&scored[idx1].0, &scored[idx2].0, &mut rng)
            };
            
            new_pop.push(child1);
            new_pop.push(child2);
        }

        // Point mutation quarter (fine-tuned changes)
        while new_pop.len() < pop_size {
            let idx = rng.gen_range(0..top_half_count);
            
            // Mix of point mutation and traditional mutation
            let mutated = if rng.gen::<f64>() < 0.6 {
                point_mutate(&scored[idx].0, &mut rng, 0.15)
            } else {
                mutate_by_index(&scored[idx].0, &mut rng, max_points)
            };
            
            new_pop.push(mutated);
        }

        population = new_pop;
    }

    // 6) Final analysis
    println!("\n=== Final Analysis ===");
    
    let mut final_scored: Vec<(UntypedAst, f64)> = population
        .into_iter()
        .map(|ast| {
            let fitness = evaluate_fitness(&mut runner, &ast, &samples);
            (ast, fitness)
        })
        .collect();
    
    final_scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    println!("Top 5 evolved programs:");
    for i in 0..5.min(final_scored.len()) {
        let (ref ast, fitness) = final_scored[i];
        let size = get_subtree_size(ast);
        println!("\n#{}: fitness={:.2}, size={}", i+1, fitness, size);
        
        // Test the program on all samples
        println!("Performance:");
        for &(x, target_y) in &samples {
            let predicted = evaluate_ast_on_x(&mut runner, ast, x);
            let error = if predicted == i32::MAX { 
                "FAIL".to_string() 
            } else { 
                format!("{}", (predicted - target_y).abs()) 
            };
            println!("  f({}) = {} (target: {}, error: {})", x, 
                if predicted == i32::MAX { "FAIL".to_string() } else { predicted.to_string() },
                target_y, error);
        }
        
        if i == 0 {
            println!("Best AST structure:");
            println!("{:#?}", ast);
        }
    }
    
    Ok(())
}