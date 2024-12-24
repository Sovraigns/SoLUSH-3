pub trait Push3Ast {
    /// Convert this AST into a bytecode array (Push3 tokens).
    fn to_bytecode(&self) -> Vec<u8>;

    /// Parse an AST from bytecode (the "decompiler"), if possible.
    /// Possibly returns a Result in case of partial/invalid parse.
    fn from_bytecode(data: &[u8]) -> Self
    where
        Self: Sized;

    /// Randomly mutate this AST in-place (GP mutation).
    fn mutate(&mut self, rng: &mut impl rand::Rng);

    /// Possibly a "crossover" method, or we do it externally.
    // fn crossover(&self, other: &Self, rng: &mut impl rand::Rng) -> Self;

    /// Evaluate the AST with some input(s).
    /// Optionally do we want this or do we always run in EVM? 
    /// We can stub it or do an interpret approach for local test.
    fn interpret_locally(&self, inputs: &[i32]) -> Vec<i32>;
}


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
    Dup,  // 0 children
    Pop,  // 0 children
}

impl Push3Ast for TypedAst {
    fn to_bytecode(&self) -> Vec<u8> {
        // do typed encoding: 
        // - for Plus(e1, e2): e1 -> bytecode, e2 -> bytecode, then PLUS opcode
        unimplemented!()
    }

    fn from_bytecode(data: &[u8]) -> Self {
        // parse in a strictly typed manner; might fail or panic 
        // if the code doesn't line up properly
        unimplemented!()
    }

    fn mutate(&mut self, rng: &mut impl rand::Rng) {
        // e.g. randomly pick a node, replace it with something 
        // that respects the typed structure
        unimplemented!()
    }

    fn interpret_locally(&self, inputs: &[i32]) -> Vec<i32> {
        // do a normal expression evaluation. If a node is "Plus(a,b)",
        // interpret (a) + interpret(b).
        unimplemented!()
    }
}

#[derive(Debug, Clone)]
pub enum UntypedAst {
    /// A single token in a list-based approach, e.g. "Integer literal"
    IntLiteral(i32),

    /// "Instruction" might be a simple variant with no children, 
    /// because the stack-based evaluation decides how many arguments it consumes.
    Instruction(OpCode),  

    /// A sublist of tokens, e.g. "( 2 3 DUP + )"
    Sublist(Vec<UntypedAst>),
}

/// Possibly an enum for OpCode:
#[derive(Debug, Clone)]
pub enum OpCode {
    Noop,
    Plus,
    Minus,
    Mult,
    Dup,
    Pop,
    // maybe Unknown => treat as no-op
}

impl Push3Ast for UntypedAst {
    fn to_bytecode(&self) -> Vec<u8> {
        // flatten out sublists vs instructions
        unimplemented!()
    }

    fn from_bytecode(data: &[u8]) -> Self {
        // parse "0x02 => INT_LITERAL => read next 4 bytes" 
        // instructions might have no subchildren
        unimplemented!()
    }

    fn mutate(&mut self, rng: &mut impl rand::Rng) {
        // can randomly insert unknown tokens, reorder sublists, etc.
        // possibly produce "invalid ops => noops"
        unimplemented!()
    }

    fn interpret_locally(&self, inputs: &[i32]) -> Vec<i32> {
        // push-based interpretation:
        // sublist => interpret each item in sequence
        // if an instruction sees insufficient args => no-op
        unimplemented!()
    }
}
