// src/bin/symreg_advanced.rs
// Advanced symbolic regression with population management

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

// Enhanced GP operators
use offchain::gp::generate_spec::ranmdom_code_fixed;
use offchain::gp::mutation::{
    point_mutate, size_aware_crossover, size_limited_mutate, get_subtree_size
};

// Advanced population management
use offchain::gp::population_management::{
    Individual, calculate_population_stats,
    diverse_elitism, apply_fitness_sharing, age_population,
    diverse_tournament_selection, calculate_novelty_score,
    enforce_minimum_diversity
};

/// Generate target function samples
fn generate_samples() -> Vec<(i32, i32)> {
    let mut samples = Vec::new();
    for x in -5..=5 {
        let y = 3 * x * x + x + 3;
        samples.push((x, y));
    }
    samples
}

/// Evaluate AST on single input
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

/// Advanced fitness function with parsimony pressure
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
    
    // Reliability bonus
    if successful_evaluations == samples.len() {
        total_fitness *= 1.2;
    }
    
    // Enhanced parsimony pressure
    let size_penalty = match get_subtree_size(ast) {
        s if s > 40 => 0.8,   // 20% penalty for very large programs
        s if s > 25 => 0.9,   // 10% penalty for large programs  
        s if s > 15 => 0.95,  // 5% penalty for medium programs
        _ => 1.0,             // No penalty for small programs
    };
    
    (total_fitness / samples.len() as f64) * size_penalty
}

fn main() -> Result<()> {
    println!("=== Advanced Population Management Evolution ===");
    
    // 1) Setup
    let samples = generate_samples();
    println!("Target function: f(x) = 3x² + x + 3");
    
    let creation_hex_filename = "../onchain/out/Push3Interpreter.sol/Push3Interpreter.json";
    let creation_bytes = get_creation_code(creation_hex_filename)?;
    let mut runner = EvmRunner::new(creation_bytes)?;

    // 2) Advanced GP parameters
    let pop_size = 300;
    let generations = 40;
    let max_points = 15;
    let max_size = 30;
    
    // Population management parameters
    let elite_ratio = 0.15;        // 15% elites
    let diversity_weight = 0.3;    // Weight for novelty in selection
    let sharing_sigma = 0.5;       // Fitness sharing radius
    let min_diversity = 0.1;       // Minimum diversity threshold
    let tournament_size = 5;       // Tournament selection size

    let elite_count = (pop_size as f64 * elite_ratio) as usize;
    let mut rng = thread_rng();

    // 3) Initialize population with diversity tracking
    let mut population: Vec<Individual> = (0..pop_size)
        .map(|_| {
            let ast = ranmdom_code_fixed(&mut rng, max_points);
            let fitness = evaluate_fitness(&mut runner, &ast, &samples);
            Individual::new(ast, fitness)
        })
        .collect();

    // Calculate initial novelty scores
    for i in 0..population.len() {
        let novelty = calculate_novelty_score(&population[i].ast, &population);
        population[i].novelty_score = novelty;
    }

    println!("\nAdvanced features enabled:");
    println!("- Diverse elitism ({}% with diversity)", (elite_ratio * 100.0) as u32);
    println!("- Fitness sharing (σ = {})", sharing_sigma);
    println!("- Novelty-based selection");
    println!("- Diversity enforcement");
    println!("- Age tracking");

    let mut stagnation_count = 0;
    let mut best_fitness_history: Vec<f64> = Vec::new();

    // 4) Advanced evolution loop
    for gen in 0..generations {
        // (a) Age population and calculate stats
        age_population(&mut population);
        
        // Update novelty scores
        for i in 0..population.len() {
            let novelty = calculate_novelty_score(&population[i].ast, &population);
            population[i].novelty_score = novelty;
        }
        
        // Apply fitness sharing to maintain diversity
        apply_fitness_sharing(&mut population, sharing_sigma);
        
        let stats = calculate_population_stats(&population);
        
        // Sort by fitness for analysis
        population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        
        let best_fitness = population[0].fitness;
        let best_size = population[0].size;
        
        println!("\n=== Generation {} ===", gen);
        println!("Best: {:.2} (size: {}, age: {})", best_fitness, best_size, population[0].age);
        println!("Population: avg={:.2}±{:.2}, diversity={:.3}", 
                 stats.avg_fitness, stats.fitness_std, stats.diversity_score);
        println!("Sizes: avg={:.1}±{:.1}", stats.avg_size, stats.size_std);
        
        // Track stagnation
        if let Some(&last_best) = best_fitness_history.last() {
            let fitness_diff = (best_fitness - last_best).abs();
            if fitness_diff < 1.0 {
                stagnation_count += 1;
            } else {
                stagnation_count = 0;
            }
        }
        best_fitness_history.push(best_fitness);
        
        if stagnation_count > 0 {
            println!("Stagnation: {} generations", stagnation_count);
        }

        // Early stopping for excellent solutions
        if best_fitness > 900.0 {
            println!("Excellent solution found! Stopping early.");
            break;
        }

        // (b) Advanced reproduction with population management
        let mut new_population = Vec::new();

        // Elite selection with diversity
        let elites = diverse_elitism(&population, elite_count, min_diversity);
        for elite in elites {
            new_population.push(elite);
        }
        println!("Elites: {} individuals selected", new_population.len());

        // Fill remainder with diverse tournament selection and advanced operators
        while new_population.len() < pop_size {
            let parent1 = diverse_tournament_selection(
                &population, tournament_size, diversity_weight, &mut rng
            );
            
            if rng.gen::<f64>() < 0.7 {
                // Crossover (70% chance)
                let parent2 = diverse_tournament_selection(
                    &population, tournament_size, diversity_weight, &mut rng
                );
                
                let (child1_ast, child2_ast) = size_aware_crossover(
                    &parent1.ast, &parent2.ast, &mut rng
                );
                
                // Evaluate children
                let child1_fitness = evaluate_fitness(&mut runner, &child1_ast, &samples);
                let child2_fitness = evaluate_fitness(&mut runner, &child2_ast, &samples);
                
                new_population.push(Individual::new(child1_ast, child1_fitness));
                if new_population.len() < pop_size {
                    new_population.push(Individual::new(child2_ast, child2_fitness));
                }
            } else {
                // Mutation (30% chance)
                let mutated_ast = if rng.gen::<f64>() < 0.6 {
                    // Point mutation (60% of mutations)
                    point_mutate(&parent1.ast, &mut rng, 0.15)
                } else {
                    // Size-limited mutation (40% of mutations)
                    size_limited_mutate(&parent1.ast, &mut rng, max_points, max_size)
                };
                
                let mutated_fitness = evaluate_fitness(&mut runner, &mutated_ast, &samples);
                new_population.push(Individual::new(mutated_ast, mutated_fitness));
            }
        }

        // (c) Enforce diversity and manage population size
        enforce_minimum_diversity(&mut new_population, min_diversity, &mut rng);
        
        // Fill back to target size if diversity enforcement removed too many
        while new_population.len() < pop_size {
            let random_ast = ranmdom_code_fixed(&mut rng, max_points);
            let random_fitness = evaluate_fitness(&mut runner, &random_ast, &samples);
            new_population.push(Individual::new(random_ast, random_fitness));
        }
        
        // Ensure exact population size
        new_population.truncate(pop_size);
        population = new_population;

        // Adaptive parameters based on diversity and stagnation
        if stats.diversity_score < 0.2 && stagnation_count > 3 {
            println!("Low diversity detected - injecting random individuals");
            // Replace worst 10% with random individuals
            let replace_count = pop_size / 10;
            population.sort_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap());
            
            for i in 0..replace_count {
                let random_ast = ranmdom_code_fixed(&mut rng, max_points);
                let random_fitness = evaluate_fitness(&mut runner, &random_ast, &samples);
                population[i] = Individual::new(random_ast, random_fitness);
            }
        }
    }

    // 5) Final analysis with detailed performance breakdown
    println!("\n=== Final Advanced Analysis ===");
    
    // Sort by fitness for final analysis
    population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
    
    let final_stats = calculate_population_stats(&population);
    println!("Final population statistics:");
    println!("  Avg fitness: {:.2} ± {:.2}", final_stats.avg_fitness, final_stats.fitness_std);
    println!("  Avg size: {:.1} ± {:.1}", final_stats.avg_size, final_stats.size_std);
    println!("  Diversity score: {:.3}", final_stats.diversity_score);
    
    println!("\nTop 3 evolved solutions:");
    for i in 0..3.min(population.len()) {
        let individual = &population[i];
        println!("\n#{}: fitness={:.2}, size={}, age={}, novelty={:.3}", 
                 i+1, individual.fitness, individual.size, individual.age, individual.novelty_score);
        
        // Detailed performance analysis
        println!("Performance breakdown:");
        let mut perfect_matches = 0;
        let mut close_matches = 0;
        let mut failures = 0;
        
        for &(x, target_y) in &samples {
            let predicted = evaluate_ast_on_x(&mut runner, &individual.ast, x);
            let status = if predicted == i32::MAX {
                failures += 1;
                "FAIL"
            } else {
                let error = (predicted - target_y).abs();
                if error == 0 {
                    perfect_matches += 1;
                    "PERFECT"
                } else if error <= 5 {
                    close_matches += 1;
                    "CLOSE"
                } else {
                    "FAR"
                }
            };
            
            println!("  f({:2}) = {:4} (target: {:2}) [{}]", 
                     x, 
                     if predicted == i32::MAX { "FAIL".to_string() } else { predicted.to_string() },
                     target_y, 
                     status);
        }
        
        println!("  Summary: {} perfect, {} close, {} failures", 
                 perfect_matches, close_matches, failures);
        
        if i == 0 {
            println!("Best solution structure:");
            println!("{:#?}", individual.ast);
        }
    }
    
    // Evolution progress summary
    println!("\nEvolution progress:");
    for (gen, &fitness) in best_fitness_history.iter().enumerate().step_by(5) {
        println!("  Gen {}: {:.2}", gen, fitness);
    }
    
    Ok(())
}