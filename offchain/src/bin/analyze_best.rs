// src/bin/analyze_best.rs
// Quick analysis of best evolved solution

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
    for x in -10..=10 {
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
                2000.0
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
        s if s > 50 => 0.7,
        s if s > 35 => 0.8,
        s if s > 25 => 0.9,
        s if s > 15 => 0.95,
        _ => 1.0,
    };
    
    (total_fitness / samples.len() as f64) * size_penalty
}

fn main() -> Result<()> {
    println!("=== Quick Best Solution Analysis ===");
    
    let samples = generate_samples();
    println!("Target function: f(x) = x³ - 2x² + 3x + 5");
    
    let creation_hex_filename = "../onchain/out/Push3Interpreter.sol/Push3Interpreter.json";
    let creation_bytes = get_creation_code(creation_hex_filename)?;
    let mut runner = EvmRunner::new(creation_bytes)?;

    // Run quick evolution
    let pop_size = 200;
    let generations = 15;
    let max_points = 15;
    let max_size = 25;
    
    let mut rng = thread_rng();
    let mut population: Vec<Individual> = (0..pop_size)
        .map(|_| {
            let ast = ranmdom_code_fixed(&mut rng, max_points);
            let fitness = evaluate_fitness(&mut runner, &ast, &samples);
            Individual::new(ast, fitness)
        })
        .collect();

    let mut best_overall: Option<Individual> = None;

    for gen in 0..generations {
        population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        
        if best_overall.is_none() || population[0].fitness > best_overall.as_ref().unwrap().fitness {
            best_overall = Some(population[0].clone());
        }
        
        println!("Gen {}: Best={:.2} (size: {})", gen, population[0].fitness, population[0].size);
        
        // Simple reproduction
        let mut new_pop = Vec::new();
        
        // Keep top 20%
        let elite_count = pop_size / 5;
        for i in 0..elite_count {
            new_pop.push(population[i].clone());
        }
        
        // Fill with mutations and crossovers
        while new_pop.len() < pop_size {
            if rng.gen::<f64>() < 0.7 {
                // Crossover
                let p1 = rng.gen_range(0..elite_count.min(50));
                let p2 = rng.gen_range(0..elite_count.min(50));
                let (child, _) = size_aware_crossover(&population[p1].ast, &population[p2].ast, &mut rng);
                let fitness = evaluate_fitness(&mut runner, &child, &samples);
                new_pop.push(Individual::new(child, fitness));
            } else {
                // Mutation
                let p = rng.gen_range(0..elite_count.min(50));
                let child = point_mutate(&population[p].ast, &mut rng, 0.15);
                let fitness = evaluate_fitness(&mut runner, &child, &samples);
                new_pop.push(Individual::new(child, fitness));
            }
        }
        
        population = new_pop;
    }

    // Analyze best solution
    if let Some(best) = best_overall {
        println!("\n=== BEST EVOLVED SOLUTION ===");
        println!("Fitness: {:.2}", best.fitness);
        println!("Size: {} nodes", best.size);
        println!("\nAST Structure:");
        println!("{:#?}", best.ast);
        
        println!("\n=== DETAILED ANALYSIS ===");
        println!("Target: f(x) = x³ - 2x² + 3x + 5");
        println!("x\tTarget\tPredicted\tError\tStatus");
        println!("─────────────────────────────────────");
        
        let mut perfect = 0;
        let mut close = 0;
        let mut failures = 0;
        let mut total_error = 0.0;
        
        for &(x, target) in &samples {
            let predicted = evaluate_ast_on_x(&mut runner, &best.ast, x);
            let status = if predicted == i32::MAX {
                failures += 1;
                "FAIL".to_string()
            } else {
                let error = (predicted - target).abs();
                total_error += error as f64;
                if error == 0 {
                    perfect += 1;
                    "PERFECT".to_string()
                } else if error <= 5 {
                    close += 1;
                    format!("CLOSE({})", error)
                } else {
                    format!("OFF({})", error)
                }
            };
            
            println!("{}\t{}\t{}\t{}\t{}", 
                     x, 
                     target, 
                     if predicted == i32::MAX { "FAIL".to_string() } else { predicted.to_string() },
                     if predicted == i32::MAX { "∞".to_string() } else { (predicted - target).abs().to_string() },
                     status);
        }
        
        let avg_error = if failures < samples.len() {
            total_error / (samples.len() - failures) as f64
        } else {
            f64::INFINITY
        };
        
        println!("\nSUMMARY:");
        println!("Perfect matches: {}/{}", perfect, samples.len());
        println!("Close matches (≤5): {}/{}", close, samples.len());
        println!("Failures: {}/{}", failures, samples.len());
        println!("Average error: {:.1}", avg_error);
        println!("Success rate: {:.1}%", (samples.len() - failures) as f64 / samples.len() as f64 * 100.0);
        
        // Try to interpret the program structure
        println!("\n=== PROGRAM INTERPRETATION ===");
        analyze_ast_structure(&best.ast, 0);
    }
    
    Ok(())
}

fn analyze_ast_structure(ast: &UntypedAst, depth: usize) {
    let indent = "  ".repeat(depth);
    
    match ast {
        UntypedAst::IntLiteral(val) => {
            println!("{}Constant: {}", indent, val);
        }
        UntypedAst::Instruction(op) => {
            let description = match op {
                offchain::compiler::ast::OpCode::Plus => "Add top two values",
                offchain::compiler::ast::OpCode::Minus => "Subtract second from top",
                offchain::compiler::ast::OpCode::Mult => "Multiply top two values",
                offchain::compiler::ast::OpCode::Dup => "Duplicate top value",
                offchain::compiler::ast::OpCode::Pop => "Remove top value",
                offchain::compiler::ast::OpCode::GreaterThan => "Push (second > top) to bool stack",
                offchain::compiler::ast::OpCode::LessThan => "Push (second < top) to bool stack",
                offchain::compiler::ast::OpCode::Equal => "Push (second == top) to bool stack",
                offchain::compiler::ast::OpCode::NotEqual => "Push (second != top) to bool stack",
                offchain::compiler::ast::OpCode::GreaterEqual => "Push (second >= top) to bool stack",
                offchain::compiler::ast::OpCode::LessEqual => "Push (second <= top) to bool stack",
                offchain::compiler::ast::OpCode::Sin => "Sine of top value",
                offchain::compiler::ast::OpCode::Cos => "Cosine of top value",
                offchain::compiler::ast::OpCode::Sqrt => "Square root of top value",
                offchain::compiler::ast::OpCode::Abs => "Absolute value of top value",
                offchain::compiler::ast::OpCode::Mod => "second % top",
                offchain::compiler::ast::OpCode::Pow => "second ^ top",
                offchain::compiler::ast::OpCode::ConstPi => "Push π (3141)",
                offchain::compiler::ast::OpCode::ConstE => "Push e (2718)",
                offchain::compiler::ast::OpCode::ConstRand => "Push random value [0,999]",
                offchain::compiler::ast::OpCode::BoolToInt => "Convert bool to int (0/1)",
                offchain::compiler::ast::OpCode::IntToBool => "Convert int to bool (0=false)",
                offchain::compiler::ast::OpCode::IfThen => "Execute next if true",
                offchain::compiler::ast::OpCode::IfElse => "Execute then/else based on condition",
                _ => "Other operation",
            };
            println!("{}Operation: {:?} - {}", indent, op, description);
        }
        UntypedAst::Sublist(children) => {
            println!("{}Block with {} operations:", indent, children.len());
            for child in children {
                analyze_ast_structure(child, depth + 1);
            }
        }
    }
}