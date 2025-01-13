//! # ast.rs
//! This module defines the **Abstract Syntax Trees (ASTs)** for our Push3-like language.
//!
//! Currently, we provide only an **untyped AST** (`UntypedAst`), which allows any sequence
//! of instructions, sublists, or integer literals. This is easy to parse and mutate, but
//! **does not** enforce correctness of operand types (for example, `INTEGER_PLUS` might
//! appear even if there's only one integer on the stack).
//!
//! ## Future: Typed AST
//! We **plan** to add a **typed AST** implementation, where each node ensures the correct
//! number and type of arguments at compile-time. This would prevent invalid programs from
//! being constructed in the first place. However, that is not implemented here yet—it’s
//! an **eventual goal** for more advanced genetic programming or verification workflows.

use serde::{Deserialize, Serialize};
// use std::fmt;

/// A trait describing the core operation: converting an AST to Push3 bytecode.
///
/// We might later add more methods here (like parsing from bytecode or mutation).
pub trait Push3Ast {
    fn to_bytecode(&self) -> Vec<u8>;
}

/// The untyped AST: each node is either an integer literal, a single instruction (no children),
/// or a sublist (which recursively contains more AST nodes).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UntypedAst {
    IntLiteral(i32),
    Instruction(OpCode),
    Sublist(Vec<UntypedAst>),
}

/// The set of push3 instructions recognized in our untyped AST.
/// Each corresponds to a single token byte in `to_bytecode`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpCode {
    Noop,
    Plus,
    Minus,
    Mult,
    Dup,
    Pop,
}

/// Implementation of `Push3Ast` for the **untyped** AST.
///
/// This produces bytecode recognized by the Push3Interpreter contract:
///
/// - `0x00` => NOOP
/// - `0x01` => INTEGER_PLUS
/// - `0x02` => INT_LITERAL => next 4 bytes (little-endian i32)
/// - `0x03` => SUBLIST => next 2 bytes (big-endian `u16` length), then the sublist contents
/// - `0x04` => INTEGER_MINUS
/// - `0x05` => INTEGER_MULT
/// - `0x06` => INTEGER_DUP
/// - `0x07` => INTEGER_POP
impl Push3Ast for UntypedAst {
    fn to_bytecode(&self) -> Vec<u8> {
        match self {
            UntypedAst::IntLiteral(val) => {
                // 0x02 => INT_LITERAL => then 4 bytes little-endian
                let mut bytes = Vec::with_capacity(1 + 4);
                bytes.push(0x02);
                bytes.extend_from_slice(&val.to_be_bytes());
                bytes
            }
            UntypedAst::Instruction(op) => {
                // Single byte representing the opcode
                let byte = match op {
                    OpCode::Noop => 0x00,
                    OpCode::Plus => 0x01,
                    OpCode::Minus => 0x04,
                    OpCode::Mult => 0x05,
                    OpCode::Dup => 0x06,
                    OpCode::Pop => 0x07,
                };
                vec![byte]
            }
            UntypedAst::Sublist(children) => {
                // 0x03 => SUBLIST
                // Then 2 bytes for the length (big-endian `u16`),
                // Then the concatenated child bytecode.
                let mut payload = Vec::new();
                for child in children {
                    let child_bytes = child.to_bytecode();
                    payload.extend_from_slice(&child_bytes);
                }

                let sub_len = payload.len() as u16;
                let mut bytes = Vec::with_capacity(1 + 2 + payload.len());
                bytes.push(0x03);
                bytes.extend_from_slice(&sub_len.to_be_bytes()); // big-endian length
                bytes.extend_from_slice(&payload);
                bytes
            }
        }
    }
}

// ----------------------------------------------------------------------------
// S-Expression Parsing Helpers
// ----------------------------------------------------------------------------

/// A simple S-expression tree for textual representation: either an `Atom` or a `List`.
#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    Atom(String),
    List(Vec<SExpr>),
}

/// Tokenize a string into parentheses and symbols (atoms).
pub fn tokenize(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    for c in s.chars() {
        match c {
            '(' | ')' => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                }
                current.clear();
                tokens.push(c.to_string());
            }
            ' ' | '\t' | '\n' | '\r' => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }

    tokens
}

/// Recursive helper to parse a single S-expression from a list of tokens, advancing `pos`.
fn parse_sexpr_internal(tokens: &[String], pos: &mut usize) -> Result<SExpr, String> {
    if *pos >= tokens.len() {
        return Err("Unexpected end of tokens".to_string());
    }

    let token = &tokens[*pos];
    match token.as_str() {
        "(" => {
            *pos += 1; // consume '('
            let mut items = Vec::new();
            while *pos < tokens.len() && tokens[*pos] != ")" {
                let expr = parse_sexpr_internal(tokens, pos)?;
                items.push(expr);
            }
            if *pos >= tokens.len() {
                return Err("Missing closing parenthesis".to_string());
            }
            *pos += 1; // consume ')'
            Ok(SExpr::List(items))
        }
        ")" => Err("Unexpected ')'".to_string()),
        _ => {
            let atom_str = token.clone();
            *pos += 1;
            Ok(SExpr::Atom(atom_str))
        }
    }
}

/// Parse a full string into a single top-level `SExpr`, assuming balanced parentheses.
pub fn parse_string_to_sexpr(s: &str) -> Result<SExpr, String> {
    let tokens = tokenize(s);
    let mut pos = 0;
    let expr = parse_sexpr_internal(&tokens, &mut pos)?;
    if pos < tokens.len() {
        Err(format!(
            "Extra tokens after parse: {:?}",
            &tokens[pos..]
        ))
    } else {
        Ok(expr)
    }
}

/// Convert an S-expression to an **untyped** AST node.
pub fn sexpr_to_untyped(expr: &SExpr) -> Result<UntypedAst, String> {
    match expr {
        SExpr::Atom(text) => {
            // 1) Try parse as integer
            if let Ok(val) = text.parse::<i32>() {
                Ok(UntypedAst::IntLiteral(val))
            } else {
                // 2) Otherwise interpret as an opcode
                match text.to_uppercase().as_str() {
                    "+" => Ok(UntypedAst::Instruction(OpCode::Plus)),
                    "-" => Ok(UntypedAst::Instruction(OpCode::Minus)),
                    "*" => Ok(UntypedAst::Instruction(OpCode::Mult)),
                    "DUP" => Ok(UntypedAst::Instruction(OpCode::Dup)),
                    "POP" => Ok(UntypedAst::Instruction(OpCode::Pop)),
                    // unknown => treat as Noop
                    _ => Ok(UntypedAst::Instruction(OpCode::Noop)),
                }
            }
        }
        SExpr::List(items) => {
            let mut sub_asts = Vec::new();
            for child in items {
                sub_asts.push(sexpr_to_untyped(child)?);
            }
            Ok(UntypedAst::Sublist(sub_asts))
        }
    }
}

// ----------------------------------------------------------------------------
// (Optional) Placeholder for a future typed AST
// ----------------------------------------------------------------------------

// /// In the future, we plan to define a `TypedAst` enum that ensures each operation
// /// has the correct number of typed children. We might also handle errors
// /// if an operation expects two integer children, but the user tries to attach
// /// a sublist or zero children, etc.
// ///
// /// For now, the untyped AST suffices for quick prototyping, but typed checks
// /// will be crucial in more advanced genetic programming or static analysis scenarios.
// #[derive(Debug, Clone)]
// pub enum TypedAst {
//     // e.g. IntLiteral(i32),
//     //     Instruction(InstructionKind),
// }

// #[derive(Debug, Clone)]
// pub enum InstructionKind {
//     // e.g. Plus(Box<TypedAst>, Box<TypedAst>),
//     //     Minus(Box<TypedAst>, Box<TypedAst>),
//     //     Mult(Box<TypedAst>, Box<TypedAst>),
//     //     Dup,
//     //     Pop,
// }

// // etc.
