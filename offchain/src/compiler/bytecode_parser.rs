use super::ast::{Expr};

pub fn bytecode_to_ast(data: &[u8]) -> Expr {
    // parse tokens, produce expr
    // stub for now
    Expr::IntLiteral(0)
}
