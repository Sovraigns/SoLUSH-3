// Population management improvements for genetic programming

use rand::Rng;
use crate::compiler::ast::UntypedAst;
use crate::gp::mutation::get_subtree_size;
// use std::collections::HashMap; // Not needed for current implementation

/// Diversity metrics and population analysis
#[derive(Debug, Clone)]
pub struct PopulationStats {
    pub avg_fitness: f64,
    pub fitness_std: f64,
    pub avg_size: f64,
    pub size_std: f64,
    pub diversity_score: f64,
    pub stagnation_count: u32,
}

/// Individual with extended information for population management
#[derive(Debug, Clone)]
pub struct Individual {
    pub ast: UntypedAst,
    pub fitness: f64,
    pub size: usize,
    pub age: u32,              // How many generations this individual has survived
    pub novelty_score: f64,    // How different this individual is from others
}

impl Individual {
    pub fn new(ast: UntypedAst, fitness: f64) -> Self {
        let size = get_subtree_size(&ast);
        Self {
            ast,
            fitness,
            size,
            age: 0,
            novelty_score: 0.0,
        }
    }
}

/// Calculate structural diversity between two ASTs
pub fn structural_distance(a: &UntypedAst, b: &UntypedAst) -> f64 {
    structural_distance_recursive(a, b, 1.0)
}

fn structural_distance_recursive(a: &UntypedAst, b: &UntypedAst, weight: f64) -> f64 {
    match (a, b) {
        (UntypedAst::IntLiteral(val_a), UntypedAst::IntLiteral(val_b)) => {
            // Distance based on value difference, normalized
            let diff = (val_a - val_b).abs() as f64;
            weight * (diff / (1.0 + diff))
        }
        (UntypedAst::Instruction(op_a), UntypedAst::Instruction(op_b)) => {
            // Binary: same instruction = 0, different = weight
            if std::mem::discriminant(op_a) == std::mem::discriminant(op_b) {
                0.0
            } else {
                weight
            }
        }
        (UntypedAst::Sublist(children_a), UntypedAst::Sublist(children_b)) => {
            // Compare sublists recursively
            let max_len = children_a.len().max(children_b.len());
            let mut total_distance = 0.0;
            
            // Size difference penalty
            let size_diff = (children_a.len() as f64 - children_b.len() as f64).abs();
            total_distance += weight * size_diff / (1.0 + max_len as f64);
            
            // Compare existing children
            let min_len = children_a.len().min(children_b.len());
            for i in 0..min_len {
                total_distance += structural_distance_recursive(
                    &children_a[i], 
                    &children_b[i], 
                    weight * 0.8  // Reduce weight for deeper nodes
                );
            }
            
            total_distance
        }
        _ => {
            // Different types = maximum distance
            weight
        }
    }
}

/// Calculate novelty score for an individual relative to population
pub fn calculate_novelty_score(individual: &UntypedAst, population: &[Individual]) -> f64 {
    if population.len() < 2 {
        return 1.0; // High novelty if population is small
    }
    
    // Find average distance to k nearest neighbors (k=5 or population_size/4)
    let k = (population.len() / 4).max(5).min(population.len() - 1);
    
    let mut distances: Vec<f64> = population
        .iter()
        .map(|other| structural_distance(individual, &other.ast))
        .collect();
    
    distances.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    // Average distance to k nearest neighbors
    let avg_distance: f64 = distances.iter().take(k).sum::<f64>() / k as f64;
    
    // Novelty is higher for individuals that are more different from their neighbors
    avg_distance
}

/// Advanced elitism that preserves diversity
pub fn diverse_elitism(
    population: &[Individual], 
    elite_count: usize,
    min_distance: f64,
) -> Vec<Individual> {
    if population.is_empty() {
        return Vec::new();
    }
    
    let mut elites = Vec::new();
    let mut remaining: Vec<Individual> = population.to_vec();
    
    // Sort by fitness (best first)
    remaining.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
    
    // Always take the best individual
    elites.push(remaining.remove(0));
    
    // For remaining elite slots, balance fitness and diversity
    while elites.len() < elite_count && !remaining.is_empty() {
        let mut best_candidate_idx = 0;
        let mut best_score = f64::NEG_INFINITY;
        
        for (i, candidate) in remaining.iter().enumerate() {
            // Calculate minimum distance to existing elites
            let min_dist_to_elites = elites
                .iter()
                .map(|elite| structural_distance(&candidate.ast, &elite.ast))
                .fold(f64::INFINITY, f64::min);
            
            // Score combines fitness and diversity
            let diversity_bonus = if min_dist_to_elites >= min_distance { 
                candidate.fitness * 0.3  // 30% bonus for being diverse
            } else { 
                0.0 
            };
            
            let total_score = candidate.fitness + diversity_bonus;
            
            if total_score > best_score {
                best_score = total_score;
                best_candidate_idx = i;
            }
        }
        
        elites.push(remaining.remove(best_candidate_idx));
    }
    
    elites
}

/// Fitness sharing to maintain diversity
pub fn apply_fitness_sharing(population: &mut [Individual], sigma: f64) {
    let n = population.len();
    
    for i in 0..n {
        let mut niche_count = 0.0;
        
        for j in 0..n {
            let distance = structural_distance(&population[i].ast, &population[j].ast);
            
            // Sharing function: 1 - (distance/sigma) if distance < sigma, else 0
            let sharing = if distance < sigma {
                1.0 - (distance / sigma)
            } else {
                0.0
            };
            
            niche_count += sharing;
        }
        
        // Adjust fitness by niche count
        if niche_count > 1.0 {
            population[i].fitness /= niche_count;
        }
    }
}

/// Age-based replacement to prevent stagnation
pub fn age_population(population: &mut [Individual]) {
    for individual in population.iter_mut() {
        individual.age += 1;
    }
}

/// Select parents using tournament selection with diversity consideration
pub fn diverse_tournament_selection<'a>(
    population: &'a [Individual],
    tournament_size: usize,
    diversity_weight: f64,
    rng: &mut impl Rng,
) -> &'a Individual {
    let tournament: Vec<&Individual> = (0..tournament_size)
        .map(|_| &population[rng.gen_range(0..population.len())])
        .collect();
    
    // Find winner based on combined fitness and novelty
    tournament
        .iter()
        .max_by(|a, b| {
            let score_a = a.fitness + diversity_weight * a.novelty_score;
            let score_b = b.fitness + diversity_weight * b.novelty_score;
            score_a.partial_cmp(&score_b).unwrap()
        })
        .unwrap()
}

/// Calculate population statistics
pub fn calculate_population_stats(population: &[Individual]) -> PopulationStats {
    if population.is_empty() {
        return PopulationStats {
            avg_fitness: 0.0,
            fitness_std: 0.0,
            avg_size: 0.0,
            size_std: 0.0,
            diversity_score: 0.0,
            stagnation_count: 0,
        };
    }
    
    let n = population.len() as f64;
    
    // Fitness statistics
    let fitnesses: Vec<f64> = population.iter().map(|ind| ind.fitness).collect();
    let avg_fitness = fitnesses.iter().sum::<f64>() / n;
    let fitness_variance = fitnesses.iter()
        .map(|f| (f - avg_fitness).powi(2))
        .sum::<f64>() / n;
    let fitness_std = fitness_variance.sqrt();
    
    // Size statistics
    let sizes: Vec<f64> = population.iter().map(|ind| ind.size as f64).collect();
    let avg_size = sizes.iter().sum::<f64>() / n;
    let size_variance = sizes.iter()
        .map(|s| (s - avg_size).powi(2))
        .sum::<f64>() / n;
    let size_std = size_variance.sqrt();
    
    // Diversity score (average pairwise distance)
    let mut total_distance = 0.0;
    let mut pair_count = 0;
    
    for i in 0..population.len() {
        for j in (i + 1)..population.len() {
            total_distance += structural_distance(&population[i].ast, &population[j].ast);
            pair_count += 1;
        }
    }
    
    let diversity_score = if pair_count > 0 {
        total_distance / pair_count as f64
    } else {
        0.0
    };
    
    PopulationStats {
        avg_fitness,
        fitness_std,
        avg_size,
        size_std,
        diversity_score,
        stagnation_count: 0, // This should be tracked externally
    }
}

/// Maintain population diversity by removing very similar individuals
pub fn enforce_minimum_diversity(
    population: &mut Vec<Individual>,
    min_distance: f64,
    rng: &mut impl Rng,
) {
    let mut to_remove = Vec::new();
    
    for i in 0..population.len() {
        for j in (i + 1)..population.len() {
            let distance = structural_distance(&population[i].ast, &population[j].ast);
            
            if distance < min_distance {
                // Remove the worse individual, or random if tied
                let remove_idx = if population[i].fitness > population[j].fitness {
                    j
                } else if population[j].fitness > population[i].fitness {
                    i
                } else {
                    // Tied fitness - remove random one
                    if rng.gen::<bool>() { i } else { j }
                };
                
                if !to_remove.contains(&remove_idx) {
                    to_remove.push(remove_idx);
                }
            }
        }
    }
    
    // Remove duplicates, sort in reverse order to maintain indices
    to_remove.sort_unstable();
    to_remove.dedup();
    
    for &idx in to_remove.iter().rev() {
        population.remove(idx);
    }
}