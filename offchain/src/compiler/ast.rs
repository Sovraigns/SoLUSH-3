#[derive(Debug, Clone)]
pub enum Expr {
    IntLiteral(i32),
    Instruction(InstructionKind, Vec<Expr>),
    // You might have a variant for sublists if needed,
    // or keep it all in the same structure.
}

#[derive(Debug, Clone)]
pub enum InstructionKind {
    // e.g. Plus, Minus, Mult, etc.
    Plus,
    Minus,
    Mult,
}
