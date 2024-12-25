// pub trait Push3Ast {
//     /// Convert this AST into a bytecode array (Push3 tokens).
//     fn to_bytecode(&self) -> Vec<u8>;

//     // Parse an AST from bytecode (the "decompiler"), if possible.
//     // Possibly returns a Result in case of partial/invalid parse.
//     // fn from_bytecode(data: &[u8]) -> Self
//     // where
//     //     Self: Sized;

//     // Randomly mutate this AST in-place (GP mutation).
//     // fn mutate(&mut self, rng: &mut impl rand::Rng);

//     // Possibly a "crossover" method, or we do it externally.
//     // fn crossover(&self, other: &Self, rng: &mut impl rand::Rng) -> Self;

//     // Evaluate the AST with some input(s).
//     // Optionally do we want this or do we always run in EVM? 
//     // We can stub it or do an interpret approach for local test.
//     // fn interpret_locally(&self, inputs: &[i32]) -> Vec<i32>;
// }


// #[derive(Debug, Clone)]
// pub enum TypedAst {
//     IntLiteral(i32),
//     Instruction(InstructionKind),
// }

// #[derive(Debug, Clone)]
// pub enum InstructionKind {
//     Plus(Box<TypedAst>, Box<TypedAst>),
//     Minus(Box<TypedAst>, Box<TypedAst>),
//     Mult(Box<TypedAst>, Box<TypedAst>),
//     Dup,  // 0 children
//     Pop,  // 0 children
// }

// impl Push3Ast for TypedAst {
//     fn to_bytecode(&self) -> Vec<u8> {
//         // do typed encoding: 
//         // - for Plus(e1, e2): e1 -> bytecode, e2 -> bytecode, then PLUS opcode
//         unimplemented!()
//     }

//     // fn from_bytecode(data: &[u8]) -> Self {
//     //     // parse in a strictly typed manner; might fail or panic 
//     //     // if the code doesn't line up properly
//     //     unimplemented!()
//     // }

//     // fn mutate(&mut self, rng: &mut impl rand::Rng) {
//     //     // e.g. randomly pick a node, replace it with something 
//     //     // that respects the typed structure
//     //     unimplemented!()
//     // }

//     // fn interpret_locally(&self, inputs: &[i32]) -> Vec<i32> {
//     //     // do a normal expression evaluation. If a node is "Plus(a,b)",
//     //     // interpret (a) + interpret(b).
//     //     unimplemented!()
//     // }
// }

// #[derive(Debug, Clone)]
// pub enum UntypedAst {
//     /// A single token in a list-based approach, e.g. "Integer literal"
//     IntLiteral(i32),

//     /// "Instruction" might be a simple variant with no children, 
//     /// because the stack-based evaluation decides how many arguments it consumes.
//     Instruction(OpCode),  

//     /// A sublist of tokens, e.g. "( 2 3 DUP + )"
//     Sublist(Vec<UntypedAst>),
// }

// /// Possibly an enum for OpCode:
// #[derive(Debug, Clone)]
// pub enum OpCode {
//     Noop,
//     Plus,
//     Minus,
//     Mult,
//     Dup,
//     Pop,
//     // maybe Unknown => treat as no-op
// }

// // impl Push3Ast for UntypedAst {
// //     fn to_bytecode(&self) -> Vec<u8> {
// //         // flatten out sublists vs instructions
// //         unimplemented!()
// //     }

// //     // fn from_bytecode(data: &[u8]) -> Self {
// //     //     // parse "0x02 => INT_LITERAL => read next 4 bytes" 
// //     //     // instructions might have no subchildren
// //     //     unimplemented!()
// //     // }

// //     // fn mutate(&mut self, rng: &mut impl rand::Rng) {
// //     //     // can randomly insert unknown tokens, reorder sublists, etc.
// //     //     // possibly produce "invalid ops => noops"
// //     //     unimplemented!()
// //     // }

// //     // fn interpret_locally(&self, inputs: &[i32]) -> Vec<i32> {
// //     //     // push-based interpretation:
// //     //     // sublist => interpret each item in sequence
// //     //     // if an instruction sees insufficient args => no-op
// //     //     unimplemented!()
// //     // }
// // }

// use crate::compiler::ast::Push3Ast;

// /// An untyped AST node, matching your Push3 structure.
// #[derive(Debug, Clone)]
// pub enum UntypedAst {
//     /// A single integer literal.
//     IntLiteral(i32),
//     /// A single push instruction (no subchildren).
//     Instruction(OpCode),
//     /// A sublist of items, each of which is another UntypedAst.
//     Sublist(Vec<UntypedAst>),
// }

// /// Your Push3 opcodes in an untyped form.
// #[derive(Debug, Clone)]
// pub enum OpCode {
//     Noop,
//     Plus,
//     Minus,
//     Mult,
//     Dup,
//     Pop,
// }

// /// Implementation of `Push3Ast` for `UntypedAst`.
// ///
// /// This will produce **bytecode** compatible with your Solidity `Push3Interpreter`:
// /// 
// /// - `0x00` => NOOP
// /// - `0x01` => INTEGER_PLUS
// /// - `0x02` => INT_LITERAL => next 4 bytes (little-endian i32)
// /// - `0x03` => SUBLIST => next 2 bytes (big-endian `u16` length), then the sublist contents
// /// - `0x04` => INTEGER_MINUS
// /// - `0x05` => INTEGER_MULT
// /// - `0x06` => INTEGER_DUP
// /// - `0x07` => INTEGER_POP
// ///
// /// In the Solidity parser, `SUBLIST (0x03)` consumes two bytes to determine
// /// how many bytes of tokens to read in that sublist.
// impl Push3Ast for UntypedAst {
//     /// Convert this untyped AST into a flattened push3 bytecode sequence.
//     fn to_bytecode(&self) -> Vec<u8> {
//         match self {
//             UntypedAst::IntLiteral(val) => {
//                 // 0x02 followed by 4 bytes of i32 (little-endian)
//                 let mut bytes = Vec::with_capacity(1 + 4);
//                 bytes.push(0x02);
//                 bytes.extend_from_slice(&val.to_le_bytes());
//                 bytes
//             }
//             UntypedAst::Instruction(op) => {
//                 // Single opcode byte
//                 let opcode_byte = match op {
//                     OpCode::Noop  => 0x00,
//                     OpCode::Plus  => 0x01,
//                     OpCode::Minus => 0x04,
//                     OpCode::Mult  => 0x05,
//                     OpCode::Dup   => 0x06,
//                     OpCode::Pop   => 0x07,
//                 };
//                 vec![opcode_byte]
//             }
//             UntypedAst::Sublist(children) => {
//                 // 0x03 => SUBLIST
//                 // Then 2 bytes for length (big-endian u16),
//                 // Then the concatenated bytecode for all children.
//                 let mut payload = Vec::new();
//                 for child in children {
//                     let child_bytes = child.to_bytecode();
//                     payload.extend_from_slice(&child_bytes);
//                 }
//                 let sub_len = payload.len() as u16; // assume it fits in 65535
//                 let mut bytes = Vec::with_capacity(1 + 2 + payload.len());
//                 bytes.push(0x03);
//                 // Store the sublist length as big-endian, because readUint16 in solidity
//                 // does `val = uint16(word >> 240)` (effectively big-endian).
//                 bytes.extend_from_slice(&sub_len.to_be_bytes());
//                 // Then the actual sublist payload
//                 bytes.extend_from_slice(&payload);
//                 bytes
//             }
//         }
//     }
// }


// #[derive(Debug, Clone, PartialEq)]
// pub enum SExpr {
//     Atom(String),          // e.g. "3", "DUP", "+"
//     List(Vec<SExpr>),      // e.g. (3 5 +)
// }

// fn tokenize(s: &str) -> Vec<String> {
//     let mut tokens = Vec::new();
//     let mut current = String::new();

//     for c in s.chars() {
//         match c {
//             '(' | ')' => {
//                 // If we were building up an atom, push it first
//                 if !current.trim().is_empty() {
//                     tokens.push(current.trim().to_string());
//                 }
//                 current.clear();
//                 // Then push the single-char token
//                 tokens.push(c.to_string());
//             }
//             ' ' | '\t' | '\n' | '\r' => {
//                 // Whitespace => if we have an atom, finish it
//                 if !current.trim().is_empty() {
//                     tokens.push(current.trim().to_string());
//                     current.clear();
//                 }
//             }
//             _ => {
//                 // Part of an atom
//                 current.push(c);
//             }
//         }
//     }

//     // If something is left in 'current' at the end
//     if !current.trim().is_empty() {
//         tokens.push(current.trim().to_string());
//     }

//     tokens
// }


// fn parse_sexpr(tokens: &[String], pos: &mut usize) -> Result<SExpr, String> {
//     // If we’re out of tokens, error
//     if *pos >= tokens.len() {
//         return Err("Unexpected end of tokens".into());
//     }

//     let token = &tokens[*pos];
//     match token.as_str() {
//         "(" => {
//             // Parse a list
//             *pos += 1; // consume '('
//             let mut list_items = Vec::new();

//             while *pos < tokens.len() && tokens[*pos] != ")" {
//                 let expr = parse_sexpr(tokens, pos)?;
//                 list_items.push(expr);
//             }

//             if *pos >= tokens.len() {
//                 return Err("Missing closing parenthesis".into());
//             }
//             // consume ")"
//             *pos += 1;
//             Ok(SExpr::List(list_items))
//         }
//         ")" => {
//             // Should never parse_sexpr starting with ')'
//             Err("Unexpected ')'".into())
//         }
//         _ => {
//             // Atom
//             let atom_str = token.clone();
//             *pos += 1;
//             Ok(SExpr::Atom(atom_str))
//         }
//     }
// }

// /// Parse an entire string into a single top-level `SExpr`.
// fn parse_string_to_sexpr(s: &str) -> Result<SExpr, String> {
//     let tokens = tokenize(s);
//     let mut pos = 0;
//     let expr = parse_sexpr(&tokens, &mut pos)?;

//     // If there's leftover tokens, optionally check
//     if pos < tokens.len() {
//         return Err(format!("Extra tokens after parse: {:?}", &tokens[pos..]));
//     }

//     Ok(expr)
// }

// fn sexpr_to_untyped(expr: &SExpr) -> Result<UntypedAst, String> {
//     match expr {
//         SExpr::Atom(text) => {
//             // 1) try to parse as i32
//             if let Ok(val) = text.parse::<i32>() {
//                 Ok(UntypedAst::IntLiteral(val))
//             } else {
//                 // 2) otherwise parse as OpCode symbol
//                 match text.to_uppercase().as_str() {
//                     "+" => Ok(UntypedAst::Instruction(OpCode::Plus)),
//                     "-" => Ok(UntypedAst::Instruction(OpCode::Minus)),
//                     "*"  => Ok(UntypedAst::Instruction(OpCode::Mult)),
//                     "DUP" => Ok(UntypedAst::Instruction(OpCode::Dup)),
//                     "POP" => Ok(UntypedAst::Instruction(OpCode::Pop)),
//                     // more opcodes ...
//                     _ => Ok(UntypedAst::Instruction(OpCode::Noop))
//                 }
//             }
//         }
//         SExpr::List(list) => {
//             // parse each child into UntypedAst
//             let mut result = Vec::new();
//             for child in list {
//                 result.push(sexpr_to_untyped(child)?);
//             }
//             Ok(UntypedAst::Sublist(result))
//         }
//     }
// }

// src/compiler/ast.rs

pub trait Push3Ast {
    /// Convert this AST into a bytecode array (Push3 tokens).
    fn to_bytecode(&self) -> Vec<u8>;
}

/* --------------------------------------------
   1) TypedAst stubs (optional)
-------------------------------------------- */

#[derive(Debug, Clone)]
pub enum TypedAst {
    IntLiteral(i32),
    Instruction(InstructionKind),
}

#[derive(Debug, Clone)]
pub enum InstructionKind {
    Plus(Box<TypedAst>, Box<TypedAst>),
    Minus(Box<TypedAst>, Box<TypedAst>),
    Mult(Box<TypedAst>, Box<TypedAst>),
    Dup,
    Pop,
}

// Example stub; not fully implemented
impl Push3Ast for TypedAst {
    fn to_bytecode(&self) -> Vec<u8> {
        unimplemented!("TypedAst::to_bytecode not yet implemented");
    }
}

/* --------------------------------------------
   2) UntypedAst + OpCode
-------------------------------------------- */

#[derive(Debug, Clone)]
pub enum UntypedAst {
    IntLiteral(i32),
    Instruction(OpCode),
    Sublist(Vec<UntypedAst>),
}

#[derive(Debug, Clone)]
pub enum OpCode {
    Noop,
    Plus,
    Minus,
    Mult,
    Dup,
    Pop,
}

/// Implementation of `Push3Ast` for `UntypedAst`.
/// Matches the Solidity-based Push3Interpreter token IDs.
impl Push3Ast for UntypedAst {
    fn to_bytecode(&self) -> Vec<u8> {
        match self {
            UntypedAst::IntLiteral(val) => {
                // 0x02 => INT_LITERAL => next 4 bytes (little-endian i32)
                let mut bytes = Vec::with_capacity(1 + 4);
                bytes.push(0x02);
                bytes.extend_from_slice(&val.to_be_bytes());
                bytes
            }
            UntypedAst::Instruction(op) => {
                let opcode_byte = match op {
                    OpCode::Noop  => 0x00,
                    OpCode::Plus  => 0x01,
                    OpCode::Minus => 0x04,
                    OpCode::Mult  => 0x05,
                    OpCode::Dup   => 0x06,
                    OpCode::Pop   => 0x07,
                };
                vec![opcode_byte]
            }
            UntypedAst::Sublist(children) => {
                // 0x03 => SUBLIST => next 2 bytes = big-endian length, then sub-payload
                let mut payload = Vec::new();
                for child in children {
                    payload.extend(child.to_bytecode());
                }
                let sub_len = payload.len() as u16;
                let mut bytes = Vec::with_capacity(1 + 2 + payload.len());
                bytes.push(0x03);
                bytes.extend_from_slice(&sub_len.to_be_bytes()); // big-endian
                bytes.extend_from_slice(&payload);
                bytes
            }
        }
    }
}

/* --------------------------------------------
   3) S-expression parsing helpers
-------------------------------------------- */
#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    Atom(String),
    List(Vec<SExpr>),
}

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

fn parse_sexpr_internal(tokens: &[String], pos: &mut usize) -> Result<SExpr, String> {
    if *pos >= tokens.len() {
        return Err("Unexpected end of tokens".into());
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
                return Err("Missing closing parenthesis".into());
            }
            *pos += 1; // consume ')'
            Ok(SExpr::List(items))
        }
        ")" => {
            Err("Unexpected ')'".into())
        }
        _ => {
            let atom = token.clone();
            *pos += 1;
            Ok(SExpr::Atom(atom))
        }
    }
}

/// Parse an entire string into a single top-level `SExpr`.
pub fn parse_string_to_sexpr(s: &str) -> Result<SExpr, String> {
    let tokens = tokenize(s);
    let mut pos = 0;
    let expr = parse_sexpr_internal(&tokens, &mut pos)?;
    if pos < tokens.len() {
        return Err(format!("Extra tokens after parse: {:?}", &tokens[pos..]));
    }
    Ok(expr)
}

/// Convert an SExpr => UntypedAst
pub fn sexpr_to_untyped(expr: &SExpr) -> Result<UntypedAst, String> {
    match expr {
        SExpr::Atom(text) => {
            if let Ok(val) = text.parse::<i32>() {
                Ok(UntypedAst::IntLiteral(val))
            } else {
                match text.to_uppercase().as_str() {
                    "+"   => Ok(UntypedAst::Instruction(OpCode::Plus)),
                    "-"   => Ok(UntypedAst::Instruction(OpCode::Minus)),
                    "*"   => Ok(UntypedAst::Instruction(OpCode::Mult)),
                    "DUP" => Ok(UntypedAst::Instruction(OpCode::Dup)),
                    "POP" => Ok(UntypedAst::Instruction(OpCode::Pop)),
                    _     => Ok(UntypedAst::Instruction(OpCode::Noop)),
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
