use anyhow::Result;
use rand::thread_rng;
use rand::Rng;

// Suppose you have these in your library:
use offchain::gp::generate_spec::ranmdom_code_fixed;
use offchain::gp::mutation::mutate_by_index;
use offchain::compiler::ast::UntypedAst;

fn main() -> Result<()> {
    let mut rng = thread_rng();

    // We'll produce, say, 3 random ASTs ("subjects").
    let num_subjects = 3;
    let max_points = 6;

    for i in 0..num_subjects {
        // Generate a random AST
        let original = ranmdom_code_fixed(&mut rng, max_points);
        println!("=== Subject #{} ===", i);
        println!("Original AST:\n{:#?}", original);

        // Let's do 2 successive mutations on it
        let mut prev = original.clone();
        for mut_idx in 0..2 {
            let mutated = mutate_by_index(&prev, &mut rng, max_points);
            println!("Mutation #{} result:\n{:#?}", mut_idx, mutated);
            prev = mutated; // so the next iteration mutates from the last result
        }

        println!("=== End of Subject #{} ===\n", i);
    }

    println!("Done generating & mutating random ASTs!");
    Ok(())
}
