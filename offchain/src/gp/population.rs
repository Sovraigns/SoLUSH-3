#[derive(Debug, Clone)]
pub struct Individual {
    pub fitness: f64,
    // your Expr or bytecode or both
}

pub struct Population {
    pub individuals: Vec<Individual>,
}
