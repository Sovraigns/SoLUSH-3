use rand::Rng;
use crate::compiler::ast::{UntypedAst, OpCode};

pub fn random_sublist_ast(rng: &mut impl Rng, max_depth: usize) -> UntypedAst {
    // 1) Choose how many children the top-level `Sublist` will have.
    let len = rng.gen_range(1..=3);

    // 2) Build a vector of sub-ASTs by calling `random_ast` for each child
    let mut children = Vec::with_capacity(len);
    for _ in 0..len {
        // We start at `depth=1` because the top-level sublist itself is `depth=0`.
        children.push(random_ast(rng, 1, max_depth));
    }

    // 3) Return a `Sublist` as the root node
    UntypedAst::Sublist(children)
}

/// Generate a random `UntypedAst` by recursively building sub-trees, 
/// int literals, or instructions. 
///
/// - `depth` tracks how deep we are in the tree.
/// - `max_depth` is the maximum allowed depth to prevent infinite recursion.
pub fn random_ast(rng: &mut impl Rng, depth: usize, max_depth: usize) -> UntypedAst {
    if depth >= max_depth {
        // Return something "terminal," 
        // e.g. an IntLiteral or a single Instruction
        random_terminal(rng)
    } else {
        // Weighted choice: 
        // 0 => IntLiteral, 
        // 1 => single Instruction,
        // 2 => Sublist with children
        let choice = rng.gen_range(0..3);
        match choice {
            0 => UntypedAst::IntLiteral(rng.gen_range(-10..10)),
            1 => UntypedAst::Instruction(random_opcode(rng)),
            2 => {
                // Make a sublist with 1..=3 children
                let len = rng.gen_range(1..=3);
                let mut children = Vec::with_capacity(len);
                for _ in 0..len {
                    children.push(random_ast(rng, depth + 1, max_depth));
                }
                UntypedAst::Sublist(children)
            }
            _ => unreachable!(),
        }
    }
}

/// Generate a random "terminal" node. 
/// Typically either an int literal or a single instruction.
fn random_terminal(rng: &mut impl Rng) -> UntypedAst {
    // Weighted choice:
    // 0 => IntLiteral
    // 1 => single Instruction
    let choice = rng.gen_range(0..2);
    match choice {
        0 => UntypedAst::IntLiteral(rng.gen_range(-10..10)),
        1 => UntypedAst::Instruction(random_opcode(rng)),
        _ => unreachable!(),
    }
}

/// Generate a random `OpCode` from your known set of instructions.
pub fn random_opcode(rng: &mut impl Rng) -> OpCode {
    let choice = rng.gen_range(0..6); 
    // because we have 6 variants: Noop, Plus, Minus, Mult, Dup, Pop
    match choice {
        0 => OpCode::Noop,
        1 => OpCode::Plus,
        2 => OpCode::Minus,
        3 => OpCode::Mult,
        4 => OpCode::Dup,
        5 => OpCode::Pop,
        _ => unreachable!(),
    }
}
