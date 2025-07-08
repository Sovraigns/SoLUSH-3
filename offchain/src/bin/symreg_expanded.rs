// src/bin/symreg_expanded.rs
// Symbolic regression with expanded instruction set

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

/// Generate target function samples - more complex polynomial for expanded testing
fn generate_samples() -> Vec<(i32, i32)> {
    let mut samples = Vec::new();
    for x in -10..=10 {
        // More complex function: f(x) = x^3 - 2*x^2 + 3*x + 5
        let y = x * x * x - 2 * x * x + 3 * x + 5;
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

/// Enhanced fitness function for expanded instruction set
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
                2000.0  // Higher reward for perfect matches
            } else if diff <= 1.0 {
                200.0 / (1.0 + diff)
            } else if diff <= 10.0 {
                100.0 / (1.0 + diff * 0.5)
            } else if diff <= 100.0 {
                50.0 / (1.0 + diff * 0.1)
            } else if diff <= 1000.0 {
                20.0 / (1.0 + diff * 0.01)
            } else {
                5.0 / (1.0 + diff * 0.001)
            };
            
            total_fitness += sample_fitness;
        }
    }
    
    // Strong reliability bonus for complex functions
    if successful_evaluations == samples.len() {
        total_fitness *= 1.5;
    } else if successful_evaluations >= samples.len() * 3 / 4 {
        total_fitness *= 1.2;
    }
    
    // Enhanced parsimony pressure for larger search space
    let size_penalty = match get_subtree_size(ast) {
        s if s > 50 => 0.7,   // 30% penalty for very large programs
        s if s > 35 => 0.8,   // 20% penalty for large programs  
        s if s > 25 => 0.9,   // 10% penalty for medium programs
        s if s > 15 => 0.95,  // 5% penalty for small-medium programs
        _ => 1.0,             // No penalty for small programs
    };
    
    (total_fitness / samples.len() as f64) * size_penalty
}

fn main() -> Result<()> {
    println!("=== Expanded Instruction Set Evolution ==");
    
    // 1) Setup
    let samples = generate_samples();
    println!("Target function: f(x) = x³ - 2x² + 3x + 5");
    println!("Sample range: x ∈ [-10, 10] ({} samples)", samples.len());
    
    let creation_hex_filename = "../onchain/out/Push3Interpreter.sol/Push3Interpreter.json";
    let creation_bytes = get_creation_code(creation_hex_filename)?;
    let mut runner = EvmRunner::new(creation_bytes)?;

    // 2) Enhanced GP parameters for expanded instruction set
    let pop_size = 400;           // Larger population for more complex search space
    let generations = 50;         // More generations for complex problems
    let max_points = 20;          // Larger programs allowed
    let max_size = 40;            // Larger size limit
    
    // Population management parameters
    let elite_ratio = 0.12;       // 12% elites
    let diversity_weight = 0.4;   // Higher weight for novelty in expanded space
    let sharing_sigma = 0.6;      // Larger sharing radius
    let min_diversity = 0.15;     // Higher minimum diversity
    let tournament_size = 7;      // Larger tournament size

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

    println!("\nExpanded instruction set features:");
    println!("- Comparison operators (>, <, ==, !=, >=, <=)");
    println!("- Mathematical functions (sin, cos, sqrt, abs, mod, pow)");
    println!("- Mathematical constants (π, e, random)");
    println!("- Type conversions (bool↔int)");
    println!("- Conditional operations (if-then, if-else)");
    println!("- Population size: {}", pop_size);
    println!("- Max program size: {} nodes", max_size);

    let mut stagnation_count = 0;
    let mut best_fitness_history: Vec<f64> = Vec::new();
    let mut best_overall_fitness = 0.0;
    let mut best_overall_ast: Option<UntypedAst> = None;

    // 4) Enhanced evolution loop
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
        
        // Track best overall solution
        if best_fitness > best_overall_fitness {
            best_overall_fitness = best_fitness;
            best_overall_ast = Some(population[0].ast.clone());
        }
        
        println!("\n=== Generation {} ===", gen);
        println!("Best: {:.2} (size: {}, age: {})", best_fitness, best_size, population[0].age);
        println!("Population: avg={:.2}±{:.2}, diversity={:.3}", 
                 stats.avg_fitness, stats.fitness_std, stats.diversity_score);
        println!("Sizes: avg={:.1}±{:.1}, best_overall={:.2}", 
                 stats.avg_size, stats.size_std, best_overall_fitness);
        
        // Track stagnation
        if let Some(&last_best) = best_fitness_history.last() {
            let fitness_diff = (best_fitness - last_best).abs();
            if fitness_diff < 2.0 {
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
        if best_fitness > 1500.0 {
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
            
            if rng.gen::<f64>() < 0.75 {
                // Crossover (75% chance - higher for complex search)
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
                // Mutation (25% chance)
                let mutated_ast = if rng.gen::<f64>() < 0.7 {
                    // Point mutation (70% of mutations)
                    point_mutate(&parent1.ast, &mut rng, 0.2)
                } else {
                    // Size-limited mutation (30% of mutations)
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
        if stats.diversity_score < 0.25 && stagnation_count > 5 {
            println!("Low diversity detected - injecting random individuals");
            // Replace worst 15% with random individuals
            let replace_count = pop_size * 15 / 100;
            population.sort_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap());
            
            for i in 0..replace_count {
                let random_ast = ranmdom_code_fixed(&mut rng, max_points);
                let random_fitness = evaluate_fitness(&mut runner, &random_ast, &samples);
                population[i] = Individual::new(random_ast, random_fitness);
            }
        }
    }

    // 5) Final analysis with expanded instruction set evaluation
    println!("\n=== Final Expanded Instruction Set Analysis ===");
    
    // Sort by fitness for final analysis
    population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
    
    let final_stats = calculate_population_stats(&population);
    println!("Final population statistics:");
    println!("  Avg fitness: {:.2} ± {:.2}", final_stats.avg_fitness, final_stats.fitness_std);
    println!("  Avg size: {:.1} ± {:.1}", final_stats.avg_size, final_stats.size_std);
    println!("  Diversity score: {:.3}", final_stats.diversity_score);
    println!("  Best overall fitness: {:.2}", best_overall_fitness);
    
    println!("\nTop 3 evolved solutions with expanded instruction set:");
    for i in 0..3.min(population.len()) {
        let individual = &population[i];
        println!("\n#{}: fitness={:.2}, size={}, age={}, novelty={:.3}", 
                 i+1, individual.fitness, individual.size, individual.age, individual.novelty_score);
        
        // Detailed performance analysis
        println!("Performance breakdown:");
        let mut perfect_matches = 0;
        let mut close_matches = 0;
        let mut failures = 0;
        let mut total_error = 0.0;
        
        for &(x, target_y) in &samples {
            let predicted = evaluate_ast_on_x(&mut runner, &individual.ast, x);
            let status = if predicted == i32::MAX {
                failures += 1;
                "FAIL"
            } else {
                let error = (predicted - target_y).abs();
                total_error += error as f64;
                if error == 0 {
                    perfect_matches += 1;
                    "PERFECT"
                } else if error <= 5 {
                    close_matches += 1;
                    "CLOSE"
                } else if error <= 50 {
                    "GOOD"
                } else {
                    "FAR"
                }
            };
            
            if i == 0 || x % 5 == 0 {  // Show details for best solution or every 5th sample
                println!("  f({:3}) = {:6} (target: {:4}) [{}]", 
                         x, 
                         if predicted == i32::MAX { "FAIL".to_string() } else { predicted.to_string() },
                         target_y, 
                         status);
            }
        }
        
        let avg_error = if failures < samples.len() {
            total_error / (samples.len() - failures) as f64
        } else {
            f64::INFINITY
        };
        
        println!("  Summary: {} perfect, {} close, {} failures, avg_error={:.1}", 
                 perfect_matches, close_matches, failures, avg_error);
        
        if i == 0 && best_overall_ast.is_some() {
            println!("Best solution structure:");
            println!("{:#?}", best_overall_ast.as_ref().unwrap());
        }
    }
    
    // Evolution progress summary
    println!("\nEvolution progress:");
    for (gen, &fitness) in best_fitness_history.iter().enumerate().step_by(10) {
        println!("  Gen {}: {:.2}", gen, fitness);
    }
    
    // Compare with theoretical target
    let target_samples: Vec<String> = samples.iter()
        .step_by(5)
        .map(|(x, y)| format!("f({})={}", x, y))
        .collect();
    println!("\nTarget function samples (every 5th): {}", target_samples.join(", "));
    
    Ok(())
}