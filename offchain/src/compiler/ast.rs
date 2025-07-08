//! src/compiler/ast.rs
//! 
//! This module defines an **abstract** untyped AST for our Push3-like language,
//! plus a mapping for how to encode opcodes as bytes. We do *not* hardcode numeric
//! values in the `OpCode` enum. Instead, we provide a trait that can map
//! opcodes to bytes. This allows us to expand or change instructions easily
//! (e.g., when the on-chain interpreter adds new opcodes or changes their IDs).

/// A trait describing how to convert an AST into Push3 bytecode.
///
/// This is deliberately minimal for now. In the future, we could add more methods
/// (like parsing from bytecode or AST mutation).
pub trait Push3Ast {
    /// Convert this AST into a bytecode vector that the on-chain interpreter
    /// can parse and execute.
    fn to_bytecode(&self) -> Vec<u8>;
}

/// A trait that **maps** each [`OpCode`] variant to its **single-byte** representation.
///
/// By centralizing this, we can easily add or reorder instructions without rewriting
/// big chunks of code everywhere. For example, if we add `OpCode::MyNewOp`, we just
/// update this trait’s logic.
pub trait OpCodeMapping {
    /// Given an `OpCode` enum variant, return the corresponding single `u8`
    /// that the interpreter expects.
    fn opcode_byte(&self, op: &OpCode) -> u8;

    // If you need reverse-lookup (byte => OpCode), you could add a method like:
    // fn from_byte(&self, b: u8) -> Option<OpCode>;
    // for now, we omit it since we only do forward mapping in this file.
}

/// Our untyped AST node:
/// - `IntLiteral(i32)` holds a literal integer,
/// - `Instruction(OpCode)` holds one opcode,
/// - `Sublist(Vec<UntypedAst>)` holds a collection of nested AST nodes.
#[derive(Debug, Clone, PartialEq)]
pub enum UntypedAst {
    IntLiteral(i32),
    Instruction(OpCode),
    Sublist(Vec<UntypedAst>),
}

/// An **abstract** set of opcodes. We do *not* assign numeric values here.
/// Instead, see [`OpCodeMapping::opcode_byte`] for how we convert them to bytes.
#[derive(Debug, Clone, PartialEq)]
pub enum OpCode {
    Noop,
    Plus,
    Minus,
    Mult,
    Dup,
    Pop,
    // Add new instructions here as needed. E.g.:
    // MyNewOp, etc.
}

impl UntypedAst {
    /// Encode this AST into bytecode, using a provided [`OpCodeMapping`].
    ///
    /// This method is more flexible than `to_bytecode()`, because you can pass in
    /// *any* mapping if needed. The method used by the trait’s `to_bytecode()`
    /// relies on the global `DEFAULT_OP_MAPPING`.
    pub fn to_bytecode_with_mapping<M: OpCodeMapping>(&self, mapping: &M) -> Vec<u8> {
        match self {
            // For an integer literal, we write the “tag byte” for int-literal, then 4 bytes (LE).
            UntypedAst::IntLiteral(val) => {
                // Hardcode 0x02 => INT_LITERAL. 
                // You *could* put that in the mapping if you want to make that flexible, too.
                let mut bytes = Vec::with_capacity(1 + 4);
                bytes.push(0x02);
                bytes.extend_from_slice(&val.to_be_bytes());
                bytes
            }
            UntypedAst::Instruction(op) => {
                // Use the mapping to find the correct opcode byte:
                let b = mapping.opcode_byte(op);
                vec![b]
            }
            UntypedAst::Sublist(children) => {
                // Hardcode 0x03 => SUBLIST, then big-endian length, then child payload
                let mut payload = Vec::new();
                for child in children {
                    let child_bytes = child.to_bytecode_with_mapping(mapping);
                    payload.extend(child_bytes);
                }
                let sub_len = payload.len() as u16;
                let mut bytes = Vec::with_capacity(1 + 2 + payload.len());
                bytes.push(0x03);
                bytes.extend_from_slice(&sub_len.to_be_bytes()); // big-endian length
                bytes.extend(payload);
                bytes
            }
        }
    }
}

/// For convenience, we implement `Push3Ast` using a *default* mapping.
impl Push3Ast for UntypedAst {
    fn to_bytecode(&self) -> Vec<u8> {
        self.to_bytecode_with_mapping(&DEFAULT_OP_MAPPING)
    }
}

/// A default mapping that corresponds to your current on-chain byte definitions.
///
/// This way, if your interpreter changes (e.g., `Minus` becomes 0x0A),
/// you just update this mapping.
pub struct DefaultOpCodeMapping;

impl OpCodeMapping for DefaultOpCodeMapping {
    fn opcode_byte(&self, op: &OpCode) -> u8 {
        match op {
            OpCode::Noop  => 0x00, // 0x00 => NOOP
            OpCode::Plus  => 0x01, // 0x01 => INTEGER_PLUS
            OpCode::Minus => 0x04, // 0x04 => INTEGER_MINUS
            OpCode::Mult  => 0x05, // 0x05 => INTEGER_MULT
            OpCode::Dup   => 0x06, // 0x06 => INTEGER_DUP
            OpCode::Pop   => 0x07, // 0x07 => INTEGER_POP
        }
    }
}

/// A convenient global `const` or `static` for quick usage.
pub const DEFAULT_OP_MAPPING: DefaultOpCodeMapping = DefaultOpCodeMapping;

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
