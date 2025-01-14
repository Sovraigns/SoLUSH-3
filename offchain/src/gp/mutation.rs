use rand::Rng;
use crate::compiler::ast::UntypedAst;
use crate::gp::generate_spec::ranmdom_code_fixed; // or push3-based generator

/// A "path" is a list of indices leading from the root to a child.
/// For example, [] is the root, [0] is the root's first child, [0,1] is that child's second child, etc.
pub type Path = Vec<usize>;

/// Enumerate all nodes in a DFS order (root first).
/// For each node, we store the `Path` (list of child indices).
pub fn enum_nodes_dfs(ast: &UntypedAst) -> Vec<Path> {
    let mut paths = Vec::new();
    dfs_helper(ast, &mut paths, &mut vec![]);
    paths
}

/// Recursive helper: `current_path` is a stack we push child indices onto.
/// `paths` is the global list of final results.
fn dfs_helper(ast: &UntypedAst, paths: &mut Vec<Path>, current_path: &mut Vec<usize>) {
    // Record the current path as a valid node
    paths.push(current_path.clone());

    match ast {
        UntypedAst::Sublist(children) => {
            // For each child, push its index, recurse, then pop
            for (i, child) in children.iter().enumerate() {
                current_path.push(i);
                dfs_helper(child, paths, current_path);
                current_path.pop();
            }
        }
        UntypedAst::IntLiteral(_) | UntypedAst::Instruction(_) => {
            // Leaf node => no children
        }
    }
}

/// Replace the node at `path` in `original` with `replacement`, returning a new AST.
/// If `path` is empty => we replace the root entirely.
pub fn replace_subtree(
    original: &UntypedAst,
    path: &[usize],
    replacement: UntypedAst
) -> UntypedAst {
    if path.is_empty() {
        // means we’re replacing the root
        return replacement;
    }
    // otherwise, we need to walk the sublist chain
    match original {
        UntypedAst::Sublist(children) => {
            // let path_head = path[0], path_tail = path[1..]
            let (first_idx, tail_path) = (path[0], &path[1..]);
            let mut new_children = children.clone();
            new_children[first_idx] = replace_subtree(&children[first_idx], tail_path, replacement);
            UntypedAst::Sublist(new_children)
        }
        // If the path is not empty but we’re not in a Sublist => error or do nothing
        UntypedAst::IntLiteral(_) | UntypedAst::Instruction(_) => {
            // Possibly, if the path is invalid, we ignore. Or we treat it as "replace anyway."
            // Here, we might treat it as an error or a no-op.
            original.clone()
        }
    }
}

/// Produces a new AST by choosing exactly one node in `original` at random
/// (by enumerating them), and replacing it with a new random subtree.
pub fn mutate_by_index(
    original: &UntypedAst,
    rng: &mut impl Rng,
    max_points: usize,
) -> UntypedAst {
    // 1) Enumerate all nodes => get a vector of `Path`
    let all_paths = enum_nodes_dfs(original);
    // pick a random path
    let idx = rng.gen_range(0..all_paths.len());
    let chosen_path = &all_paths[idx];

    // 2) Generate a new subtree with our "fixed" random code generator
    let new_subtree = ranmdom_code_fixed(rng, max_points);

    // 3) Replace the subtree at `chosen_path` in `original` with `new_subtree`
    replace_subtree(original, chosen_path, new_subtree)
}

