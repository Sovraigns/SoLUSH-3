use rand::Rng;
use crate::compiler::ast::{UntypedAst, OpCode};
use crate::gp::generate::{random_ast, random_opcode};

/// Returns a new `UntypedAst` that is a mutated version of `original`.
/// We do not modify `original`; we clone or build new pieces as needed.
/// 
/// - `depth`: current depth of this node,
/// - `max_depth`: max allowed depth (to prevent infinite recursion).
pub fn mutated_ast(original: &UntypedAst, rng: &mut impl Rng, depth: usize, max_depth: usize) -> UntypedAst {
    // 10% chance: completely replace this node with a fresh random subtree
    if rng.gen_range(0..100) < 10 {
        return random_ast(rng, depth, max_depth);
    }

    match original {
        UntypedAst::IntLiteral(val) => {
            // Another small chance to replace the entire node with random subtree
            if rng.gen_range(0..100) < 10 {
                return random_ast(rng, depth, max_depth);
            }

            // Otherwise, "tweak" the int by a small offset
            let delta = rng.gen_range(-2..=2);
            let new_val = val.saturating_add(delta);
            UntypedAst::IntLiteral(new_val)
        }
        UntypedAst::Instruction(op) => {
            // 10% chance to replace node
            if rng.gen_range(0..100) < 10 {
                return random_ast(rng, depth, max_depth);
            }

            // 50% chance to pick a new opcode
            // or else keep the same
            if rng.gen_bool(0.5) {
                UntypedAst::Instruction(random_opcode(rng))
            } else {
                // Just clone the original
                UntypedAst::Instruction(op.clone())
            }
        }
        UntypedAst::Sublist(children) => {
            // 10% chance to replace this sublist entirely
            if rng.gen_range(0..100) < 10 {
                return random_ast(rng, depth, max_depth);
            }

            // Otherwise, mutate zero or more children
            let mut new_children = children.clone();
            if !new_children.is_empty() {
                // Choose how many kids to mutate: 
                // e.g., 1..=min(2, children.len())
                let mut_count = rng.gen_range(1..=new_children.len().min(2));

                for _ in 0..mut_count {
                    let idx = rng.gen_range(0..new_children.len());
                    new_children[idx] = mutated_ast(&new_children[idx], rng, depth+1, max_depth);
                }
            }
            UntypedAst::Sublist(new_children)
        }
    }
}
