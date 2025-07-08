//! src/gp/local_mutation.rs
//! A localized mutation operator that makes small tweaks depending on node type.

use rand::Rng;
use crate::compiler::ast::{UntypedAst, OpCode};
use crate::gp::mutation::{
    enum_nodes_dfs, get_subtree, replace_subtree,
};
use crate::gp::generate_spec::{
    InstructionAtom, InstructionSet,
};

/// A top-level function to perform a localized mutation with a chosen `InstructionSet`.
pub fn local_mutation(
    original: &UntypedAst,
    rng: &mut impl Rng,
    instr_set: &InstructionSet,
) -> UntypedAst {
    let all_paths = enum_nodes_dfs(original);
    let idx = rng.gen_range(0..all_paths.len());
    let chosen_path = &all_paths[idx];

    let old_subtree = get_subtree(original, chosen_path);
    let new_subtree = local_mutation_node(&old_subtree, rng, instr_set);
    replace_subtree(original, chosen_path, new_subtree)
}

/// A convenience wrapper that always uses `InstructionSet::new_default()`.
pub fn local_mutation_fixed(
    original: &UntypedAst,
    rng: &mut impl Rng,
) -> UntypedAst {
    let instr_set = InstructionSet::new_default();
    local_mutation(original, rng, &instr_set)
}

fn local_mutation_node(
    subtree: &UntypedAst,
    rng: &mut impl Rng,
    instr_set: &InstructionSet,
) -> UntypedAst {
    match subtree {
        UntypedAst::IntLiteral(val) => {
            let delta = rng.gen_range(-5..=5);
            let new_val = val.saturating_add(delta);
            UntypedAst::IntLiteral(new_val)
        }
        UntypedAst::Instruction(op) => {
            let mut new_op = pick_random_opcode(rng, instr_set);
            for _ in 0..3 {
                if &new_op != op {
                    break;
                }
                new_op = pick_random_opcode(rng, instr_set);
            }
            UntypedAst::Instruction(new_op)
        }
        UntypedAst::Sublist(children) => {
            let choice = rng.gen_range(0..3);
            let mut new_children = children.clone();

            match choice {
                0 => {
                    // remove one child if possible
                    if !new_children.is_empty() {
                        let i = rng.gen_range(0..new_children.len());
                        new_children.remove(i);
                    }
                }
                1 => {
                    // insert a new small node
                    let i = rng.gen_range(0..=new_children.len());
                    let node = create_small_node(rng, instr_set);
                    new_children.insert(i, node);
                }
                2 => {
                    // reorder two children
                    if new_children.len() > 1 {
                        let i = rng.gen_range(0..new_children.len());
                        let j = rng.gen_range(0..new_children.len());
                        new_children.swap(i, j);
                    }
                }
                _ => {}
            }

            UntypedAst::Sublist(new_children)
        }
    }
}

fn pick_random_opcode(rng: &mut impl Rng, instr_set: &InstructionSet) -> OpCode {
    use crate::compiler::ast::OpCode;

    let mut opcode_list: Vec<&OpCode> = Vec::new();
    for atom in &instr_set.atoms {
        if let InstructionAtom::Opcode(ref op) = atom {
            opcode_list.push(op);
        }
    }
    if opcode_list.is_empty() {
        // fallback => default to Noop
        return OpCode::Noop;
    }
    let i = rng.gen_range(0..opcode_list.len());
    opcode_list[i].clone()
}

fn create_small_node(
    rng: &mut impl Rng,
    instr_set: &InstructionSet,
) -> UntypedAst {
    instr_set.random_atom_as_ast(rng)
}
