// src/bin/quick_analysis.rs
// Quick analysis of a specific evolved solution

use anyhow::Result;
use rand::thread_rng;

// Our GP + compiler modules 
use offchain::compiler::ast::{UntypedAst, Push3Ast, OpCode};
use offchain::helpers::artifact::get_creation_code;
use offchain::runner::revm_runner::{
    EvmRunner, 
    Push3InterpreterInputs,
};
use offchain::compiler::push3_describtor::make_sublist_descriptor;
use offchain::gp::generate_spec::ranmdom_code_fixed;
use offchain::gp::mutation::get_subtree_size;

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
fn evaluate_ast_on_x(runner: &mut EvmRunner, ast: &UntypedAst, x: i32) -> i32 {
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

    match runner.run_interpreter(&inputs) {
        Ok(outputs) => {
            if outputs.final_int_stack.is_empty() {
                i32::MAX
            } else {
                *outputs.final_int_stack.last().unwrap() as i32
            }
        }
        Err(_) => i32::MAX,
    }
}

fn main() -> Result<()> {
    println!("=== Analyzing Hand-Picked Best Solution ===");
    
    let samples = generate_samples();
    println!("Target function: f(x) = x³ - 2x² + 3x + 5");
    
    let creation_hex_filename = "../onchain/out/Push3Interpreter.sol/Push3Interpreter.json";
    let creation_bytes = get_creation_code(creation_hex_filename)?;
    let mut runner = EvmRunner::new(creation_bytes)?;

    // Generate a few candidates and analyze the best performing one
    let mut best_ast: Option<UntypedAst> = None;
    let mut best_fitness = 0.0;
    let mut rng = thread_rng();
    
    println!("Testing 1000 random candidates...");
    
    for i in 0..1000 {
        let ast = ranmdom_code_fixed(&mut rng, 12);
        
        let mut total_error = 0.0;
        let mut failures = 0;
        
        for &(x, target) in &samples {
            let predicted = evaluate_ast_on_x(&mut runner, &ast, x);
            if predicted == i32::MAX {
                failures += 1;
                total_error += 1000.0; // Heavy penalty for failures
            } else {
                total_error += (predicted - target).abs() as f64;
            }
        }
        
        // Simple fitness: lower error = higher fitness
        let fitness = if failures == samples.len() {
            0.1
        } else {
            1000.0 / (1.0 + total_error / samples.len() as f64)
        };
        
        if fitness > best_fitness {
            best_fitness = fitness;
            best_ast = Some(ast);
            
            if i % 100 == 0 {
                println!("Candidate {}: New best fitness {:.2}", i, fitness);
            }
        }
    }
    
    if let Some(ast) = best_ast {
        println!("\n=== BEST SOLUTION FOUND ===");
        println!("Fitness: {:.2}", best_fitness);
        println!("Size: {} nodes", get_subtree_size(&ast));
        
        println!("\n=== AST STRUCTURE ===");
        println!("{:#?}", ast);
        
        println!("\n=== PERFORMANCE ANALYSIS ===");
        println!("x\tTarget\tPredicted\tError\tStatus");
        println!("─────────────────────────────────────");
        
        let mut perfect = 0;
        let mut close = 0;
        let mut failures = 0;
        let mut total_error = 0.0;
        
        for &(x, target) in &samples {
            let predicted = evaluate_ast_on_x(&mut runner, &ast, x);
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
                } else if error <= 20 {
                    format!("GOOD({})", error)
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
        
        println!("\n=== SUMMARY ===");
        println!("Perfect matches: {}/{}", perfect, samples.len());
        println!("Close matches (≤5): {}/{}", close, samples.len());
        println!("Good matches (≤20): {}/{}", 
                 samples.iter().map(|&(x, target)| {
                     let predicted = evaluate_ast_on_x(&mut runner, &ast, x);
                     if predicted != i32::MAX && (predicted - target).abs() <= 20 { 1 } else { 0 }
                 }).sum::<i32>(), samples.len());
        println!("Failures: {}/{}", failures, samples.len());
        println!("Average error: {:.1}", avg_error);
        println!("Success rate: {:.1}%", (samples.len() - failures) as f64 / samples.len() as f64 * 100.0);
        
        println!("\n=== PROGRAM INTERPRETATION ===");
        analyze_ast_structure(&ast, 0);
        
        // Show what the program computes for a few key points
        println!("\n=== FUNCTION BEHAVIOR ===");
        for x in [-5, -1, 0, 1, 3, 5] {
            let target = x * x * x - 2 * x * x + 3 * x + 5;
            let predicted = evaluate_ast_on_x(&mut runner, &ast, x);
            if predicted != i32::MAX {
                println!("f({}) = {} (target: {}, error: {})", x, predicted, target, (predicted - target).abs());
            }
        }
    } else {
        println!("No good solution found in random search");
    }
    
    Ok(())
}

fn analyze_ast_structure(ast: &UntypedAst, depth: usize) {
    let indent = "  ".repeat(depth);
    
    match ast {
        UntypedAst::IntLiteral(val) => {
            println!("{}📊 Constant: {}", indent, val);
        }
        UntypedAst::Instruction(op) => {
            let (symbol, description) = match op {
                OpCode::Plus => ("➕", "Add top two values"),
                OpCode::Minus => ("➖", "Subtract second from top"),
                OpCode::Mult => ("✖️", "Multiply top two values"),
                OpCode::Dup => ("📋", "Duplicate top value"),
                OpCode::Pop => ("🗑️", "Remove top value"),
                OpCode::GreaterThan => ("🔍>", "Push (second > top) to bool stack"),
                OpCode::LessThan => ("🔍<", "Push (second < top) to bool stack"),
                OpCode::Equal => ("🔍=", "Push (second == top) to bool stack"),
                OpCode::NotEqual => ("🔍≠", "Push (second != top) to bool stack"),
                OpCode::Abs => ("📏", "Absolute value of top"),
                OpCode::Sqrt => ("√", "Square root of top"),
                OpCode::Pow => ("^", "second raised to power of top"),
                OpCode::Mod => ("%", "second modulo top"),
                OpCode::Sin => ("sin", "Sine of top value"),
                OpCode::Cos => ("cos", "Cosine of top value"),
                OpCode::ConstPi => ("π", "Push π (3141)"),
                OpCode::ConstE => ("e", "Push e (2718)"),
                OpCode::ConstRand => ("🎲", "Push random [0,999]"),
                OpCode::BoolToInt => ("bool→int", "Convert bool to 0/1"),
                OpCode::IntToBool => ("int→bool", "Convert int to bool"),
                OpCode::IfThen => ("if", "Execute next if true"),
                OpCode::IfElse => ("if-else", "Branch execution"),
                _ => ("?", "Other operation"),
            };
            println!("{}{} {} - {}", indent, symbol, format!("{:?}", op), description);
        }
        UntypedAst::Sublist(children) => {
            println!("{}📦 Block with {} operations:", indent, children.len());
            for child in children {
                analyze_ast_structure(child, depth + 1);
            }
        }
    }
}