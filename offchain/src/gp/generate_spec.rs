use crate::compiler::ast::{UntypedAst, OpCode};
use rand::Rng;
use rand::prelude::SliceRandom;

#[derive(Clone, Debug)]
pub enum InstructionAtom {
    /// A normal opcode, e.g. `Plus`, `Minus`, `Dup`, etc.
    Opcode(OpCode),
    /// An ephemeral random constant placeholder. 
    /// If chosen, we generate a random literal on the spot.
    EphemeralInt,
    // If you want ephemeral floats, booleans, etc., add more variants
}

/// A small struct to hold our entire “instruction set.”
#[derive(Clone, Debug)]
pub struct InstructionSet {
    pub atoms: Vec<InstructionAtom>,
}

impl InstructionSet {
    pub fn new_default() -> Self {
        // Example: Some opcodes + ephemeral constant
        // Tweak as you like
        use InstructionAtom::*;
        use OpCode::*;

        Self {
            atoms: vec![
                // Basic operations
                Opcode(Noop),
                Opcode(Plus),
                Opcode(Minus),
                Opcode(Mult),
                Opcode(Dup),
                Opcode(Pop),
                
                // Comparison operations
                Opcode(GreaterThan),
                Opcode(LessThan),
                Opcode(Equal),
                Opcode(NotEqual),
                Opcode(GreaterEqual),
                Opcode(LessEqual),
                
                // Mathematical functions
                Opcode(Sin),
                Opcode(Cos),
                Opcode(Sqrt),
                Opcode(Abs),
                Opcode(Mod),
                Opcode(Pow),
                
                // Constants
                Opcode(ConstPi),
                Opcode(ConstE),
                Opcode(ConstRand),
                
                // Type conversions
                Opcode(BoolToInt),
                Opcode(IntToBool),
                
                // Conditional operations
                Opcode(IfThen),
                Opcode(IfElse),
                
                // Ephemeral constants
                EphemeralInt,
            ],
        }
    }
    
    /// Pick a random atom from this set.
    /// If it's `EphemeralInt`, we produce `UntypedAst::IntLiteral(...)`.
    /// If it's `Opcode(...)`, we produce `UntypedAst::Instruction(...)`.
    pub fn random_atom_as_ast(&self, rng: &mut impl Rng) -> UntypedAst {
        let idx = rng.gen_range(0..self.atoms.len());
        match &self.atoms[idx] {
            InstructionAtom::Opcode(op) => UntypedAst::Instruction(op.clone()),
            InstructionAtom::EphemeralInt => {
                // For ephemeral int, produce a random literal in some range
                let val = rng.gen_range(-30..30);
                UntypedAst::IntLiteral(val)
            }
        }
    }
}

pub fn ranmdom_code_fixed(rng: &mut impl Rng, max_points: usize) -> UntypedAst {
    let instr_set = InstructionSet::new_default();
    return random_code(rng, &instr_set, max_points);
}

/// The "entry point": random_code(max_points).
/// 1) Choose actual_points in [1..=max_points].
/// 2) Return the result of random_code_with_size.
pub fn random_code(rng: &mut impl Rng, instr_set: &InstructionSet, max_points: usize) -> UntypedAst {
    let actual_points = rng.gen_range(1..=max_points);
    random_code_with_size(rng, instr_set, actual_points)
}

/// The main logic: 
///   If points == 1 => pick a single instruction/ephemeral 
///   Else => 
///       - We "decompose" (points - 1) among sub-codes
///       - For each sub-points, we recursively call random_code_with_size
///       - We return a Sublist of those children, possibly in random order
pub fn random_code_with_size(
    rng: &mut impl Rng,
    instr_set: &InstructionSet,
    points: usize,
) -> UntypedAst {
    use UntypedAst::*;

    // Base case
    if points == 1 {
        // Choose a random "atom" from instr_set
        return instr_set.random_atom_as_ast(rng);
    } 

    // If `points > 1`, let's produce a Sublist
    // We'll break (points - 1) into sub-points via `decompose`
    let subpoints_list = decompose(rng, points - 1, points - 1);
    // subpoints_list is e.g. [2, 3, 5] and sums to (points-1)

    // Then for each "subpoints", we do random_code_with_size 
    let mut sub_asts: Vec<UntypedAst> = subpoints_list
        .into_iter()
        .map(|sp| random_code_with_size(rng, instr_set, sp))
        .collect();

    // The spec says "Return a list containing the results, in random order"
    // so let's shuffle sub_asts
    sub_asts.shuffle(rng);

    Sublist(sub_asts)
}

/// Decompose a number into random parts. 
/// 
///  - If number is 1 or max_parts is 1 => return [number].
///  - Otherwise pick a random split "this_part" and recurse on the remainder.
fn decompose(rng: &mut impl Rng, number: usize, max_parts: usize) -> Vec<usize> {
    if number == 1 || max_parts == 1 {
        return vec![number];
    }
    // pick a random "this_part" in [1 .. (number-1)]
    let this_part = rng.gen_range(1..number);
    let mut remainder = decompose(rng, number - this_part, max_parts - 1);
    let mut result = vec![this_part];
    result.append(&mut remainder);
    result
}
