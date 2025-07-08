use rand::Rng;
use crate::compiler::ast::{UntypedAst, OpCode};
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

/// Return the subtree of `original` at `path`, 
/// cloning it as a `UntypedAst`. 
/// If `path` is empty => returns the entire `original`.
pub fn get_subtree(original: &UntypedAst, path: &[usize]) -> UntypedAst {
    if path.is_empty() {
        // The entire AST is the subtree
        return original.clone();
    }

    match original {
        UntypedAst::Sublist(children) => {
            let (first_idx, tail_path) = (path[0], &path[1..]);
            if first_idx >= children.len() {
                // If path index is out of range, just clone the entire AST or 
                // do something fallback
                return original.clone();
            }
            get_subtree(&children[first_idx], tail_path)
        }
        UntypedAst::IntLiteral(_) | UntypedAst::Instruction(_) => {
            // If path is non-empty but this is a leaf, 
            // fallback: just return the leaf.
            original.clone()
        }
    }
}

/// Perform a subtree crossover between two ASTs.
/// 1) We pick a random node in `a` and a random node in `b`,
/// 2) We swap those subtrees,
/// 3) Return the two new ASTs.
///
/// Example usage:
/// ```rust
/// let (childA, childB) = crossover_by_index(&parentA, &parentB, &mut rng);
/// ```
pub fn crossover_by_index(
    a: &UntypedAst,
    b: &UntypedAst,
    rng: &mut impl Rng,
) -> (UntypedAst, UntypedAst) {
    // 1) enumerate nodes in a
    let paths_a = enum_nodes_dfs(a);
    let idx_a = rng.gen_range(0..paths_a.len());
    let chosen_a = &paths_a[idx_a];

    // 2) enumerate nodes in b
    let paths_b = enum_nodes_dfs(b);
    let idx_b = rng.gen_range(0..paths_b.len());
    let chosen_b = &paths_b[idx_b];

    // 3) get subtree from a, from b
    let subtree_a = get_subtree(a, chosen_a);
    let subtree_b = get_subtree(b, chosen_b);

    // 4) replace them
    let new_a = replace_subtree(a, chosen_a, subtree_b);
    let new_b = replace_subtree(b, chosen_b, subtree_a);

    (new_a, new_b)
}

/// Point mutation: Make small changes to individual nodes
/// This is less destructive than subtree mutation
pub fn point_mutate(
    original: &UntypedAst,
    rng: &mut impl Rng,
    mutation_rate: f64,
) -> UntypedAst {
    point_mutate_recursive(original, rng, mutation_rate)
}

fn point_mutate_recursive(
    ast: &UntypedAst,
    rng: &mut impl Rng,
    mutation_rate: f64,
) -> UntypedAst {
    // Decide if this node gets mutated
    let should_mutate = rng.gen::<f64>() < mutation_rate;
    
    match ast {
        UntypedAst::IntLiteral(val) => {
            if should_mutate {
                // Small integer mutations: add/subtract small random value
                let delta = rng.gen_range(-5..=5);
                let new_val = val.saturating_add(delta);
                UntypedAst::IntLiteral(new_val)
            } else {
                ast.clone()
            }
        }
        UntypedAst::Instruction(op) => {
            if should_mutate {
                // Mutate to a different instruction
                let new_op = match rng.gen_range(0..6) {
                    0 => OpCode::Noop,
                    1 => OpCode::Plus,
                    2 => OpCode::Minus,
                    3 => OpCode::Mult,
                    4 => OpCode::Dup,
                    5 => OpCode::Pop,
                    _ => op.clone(),
                };
                UntypedAst::Instruction(new_op)
            } else {
                ast.clone()
            }
        }
        UntypedAst::Sublist(children) => {
            // Recursively apply point mutation to children
            let new_children: Vec<UntypedAst> = children
                .iter()
                .map(|child| point_mutate_recursive(child, rng, mutation_rate))
                .collect();
            
            // Possibly add/remove children (structural mutation)
            if should_mutate && rng.gen::<f64>() < 0.3 {
                let mut modified_children = new_children;
                
                if modified_children.len() > 1 && rng.gen::<bool>() {
                    // Remove a random child (10% chance)
                    let remove_idx = rng.gen_range(0..modified_children.len());
                    modified_children.remove(remove_idx);
                } else if modified_children.len() < 8 {
                    // Add a simple random child (20% chance)
                    let new_child = if rng.gen::<bool>() {
                        UntypedAst::IntLiteral(rng.gen_range(-10..=10))
                    } else {
                        UntypedAst::Instruction(match rng.gen_range(0..5) {
                            0 => OpCode::Plus,
                            1 => OpCode::Minus,
                            2 => OpCode::Mult,
                            3 => OpCode::Dup,
                            _ => OpCode::Pop,
                        })
                    };
                    let insert_idx = rng.gen_range(0..=modified_children.len());
                    modified_children.insert(insert_idx, new_child);
                }
                
                UntypedAst::Sublist(modified_children)
            } else {
                UntypedAst::Sublist(new_children)
            }
        }
    }
}

/// Size-aware crossover: prefer swapping subtrees of similar sizes
pub fn size_aware_crossover(
    a: &UntypedAst,
    b: &UntypedAst,
    rng: &mut impl Rng,
) -> (UntypedAst, UntypedAst) {
    let paths_a = enum_nodes_dfs(a);
    let paths_b = enum_nodes_dfs(b);
    
    // Get sizes of all subtrees
    let sizes_a: Vec<usize> = paths_a.iter()
        .map(|path| get_subtree_size(&get_subtree(a, path)))
        .collect();
    let sizes_b: Vec<usize> = paths_b.iter()
        .map(|path| get_subtree_size(&get_subtree(b, path)))
        .collect();
    
    // Try to find subtrees of similar size (within 2x factor)
    let mut best_pair = None;
    let mut best_size_ratio = f64::INFINITY;
    
    for (i, &size_a) in sizes_a.iter().enumerate() {
        for (j, &size_b) in sizes_b.iter().enumerate() {
            let ratio = if size_a >= size_b {
                size_a as f64 / size_b as f64
            } else {
                size_b as f64 / size_a as f64
            };
            
            if ratio < best_size_ratio {
                best_size_ratio = ratio;
                best_pair = Some((i, j));
            }
        }
    }
    
    // Use best pair if found, otherwise fall back to random
    let (idx_a, idx_b) = best_pair.unwrap_or_else(|| {
        (rng.gen_range(0..paths_a.len()), rng.gen_range(0..paths_b.len()))
    });
    
    let chosen_a = &paths_a[idx_a];
    let chosen_b = &paths_b[idx_b];
    
    let subtree_a = get_subtree(a, chosen_a);
    let subtree_b = get_subtree(b, chosen_b);
    
    let new_a = replace_subtree(a, chosen_a, subtree_b);
    let new_b = replace_subtree(b, chosen_b, subtree_a);
    
    (new_a, new_b)
}

/// Calculate the size (number of nodes) of an AST
pub fn get_subtree_size(ast: &UntypedAst) -> usize {
    match ast {
        UntypedAst::IntLiteral(_) | UntypedAst::Instruction(_) => 1,
        UntypedAst::Sublist(children) => {
            1 + children.iter().map(get_subtree_size).sum::<usize>()
        }
    }
}

/// Size-limited mutation: prevents excessive growth
pub fn size_limited_mutate(
    original: &UntypedAst,
    rng: &mut impl Rng,
    max_points: usize,
    max_size: usize,
) -> UntypedAst {
    let current_size = get_subtree_size(original);
    
    // If already too large, try to shrink
    if current_size > max_size {
        return shrink_ast(original, rng, max_size);
    }
    
    // Otherwise, use regular mutation but check size
    let mutated = mutate_by_index(original, rng, max_points);
    let new_size = get_subtree_size(&mutated);
    
    if new_size <= max_size {
        mutated
    } else {
        // Mutation made it too large, try point mutation instead
        point_mutate(original, rng, 0.1)
    }
}

/// Shrink an AST by removing nodes/subtrees
fn shrink_ast(ast: &UntypedAst, rng: &mut impl Rng, target_size: usize) -> UntypedAst {
    match ast {
        UntypedAst::IntLiteral(_) | UntypedAst::Instruction(_) => ast.clone(),
        UntypedAst::Sublist(children) => {
            if children.is_empty() {
                return ast.clone();
            }
            
            // Remove a random child
            let mut new_children = children.clone();
            if new_children.len() > 1 {
                let remove_idx = rng.gen_range(0..new_children.len());
                new_children.remove(remove_idx);
            }
            
            let result = UntypedAst::Sublist(new_children);
            
            // Recursively shrink if still too large
            if get_subtree_size(&result) > target_size && !children.is_empty() {
                shrink_ast(&result, rng, target_size)
            } else {
                result
            }
        }
    }
}
