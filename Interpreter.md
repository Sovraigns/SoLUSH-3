# Interpreter.md

## Overview

This repository provides a **Push3-like interpreter** implemented in Solidity. It uses a _token-based bytecode_ approach to parse programs from a `bytes` array (the `code`) into typed **descriptors**, which are then executed by the interpreter.

**Push3** is a stack-based language originally designed for genetic programming and autoconstructive evolution. Each data type (e.g. `INT_LITERAL`, `FLOAT`, `BOOLEAN`, etc.) has its own stack. The **`EXEC`** stack manages the flow of execution.

**In this implementation**:
- We store code in a single `bytes` array (`code`).
- We parse sublists and integer literals from that array.
- We push instructions and literals onto an execution stack.
- We pop them in LIFO order and execute them.

### Key Features

1. **Token-Based Parsing**: We define small tokens (`0x00` => NOOP, `0x01` => INTEGER_PLUS, `0x02` => INT_LITERAL, `0x03` => SUBLIST). If we see `INT_LITERAL`, we read 4 bytes for a 32-bit integer; if we see `SUBLIST`, we read 2 bytes for the sublist length, then parse it recursively.
2. **Descriptor System**: We represent instructions, literals, and sublists using 256-bit “descriptors.” Each descriptor encodes:
   - A `tag` (top 8 bits): e.g. `INT_LITERAL`, `INSTRUCTION`, `SUBLIST`.
   - `offset` and `length` for sublists.
   - A leftover field (e.g. for storing integer data).
3. **Reversing Items**: When we pop a sublist descriptor, we parse it into descriptors and then push them onto the `EXEC` stack in reverse order. This ensures the list’s leftmost token executes first, consistent with the Push3 spec.
4. **Separation of Parsing and Execution**: 
   - **Parsing**: Transforms raw byte tokens into typed descriptors (instructions, literals, or nested sublists).
   - **Execution**: The `runInterpreter` function repeatedly pops from `EXEC`, decides if it’s an instruction or literal or sublist, and acts accordingly.

## Code Structure

### 1. **Enums**

- **`CodeTag`**: 
  - `NO_TAG` (unused), 
  - `INSTRUCTION`, 
  - `INT_LITERAL`, 
  - `SUBLIST`.

- **`OpCode`**:  
  - `NOOP`, 
  - `INTEGER_PLUS`.

### 2. **Descriptor Layout**

A **256-bit** word has:
```
[  top 8 bits    |  offset(32 bits) |  length(32 bits)  |  leftover(184 bits) ]
```
- `tag`: The top 8 bits (e.g. `INT_LITERAL`).
- `offset, length`: If `tag = SUBLIST`, these define which bytes in `code` represent the sub-sublists.
- `leftover`: If `tag = INT_LITERAL`, we store the 32-bit integer in the bottom 32 bits of leftover. If `tag = INSTRUCTION`, we store the opcode (`NOOP`, `INTEGER_PLUS`) in the lowest 8 bits of leftover.

### 3. **Helper Reads**

- **`readUint32(code, start)`**: Copies 4 bytes from calldata into a small buffer, returns the final `uint32`. By shifting bits if needed, we ensure we get the integer from the correct part of the loaded word.
- **`readUint16(code, start)`**: Similar logic for 2 bytes.

### 4. **`parseSublist` Function**

When we see a sublist descriptor (`tag=SUBLIST`), we parse the slice `[off..off+len]` in the `bytes code`. We iterate token by token:
- `0x00` => `NOOP`
- `0x01` => `INTEGER_PLUS`
- `0x02` => read 4 bytes => `INT_LITERAL`
- `0x03` => read 2 bytes => sub-sublist length => build a sub-descriptor

We accumulate these descriptors in an array. Then, later, we push them _in reverse_ onto the `EXEC` stack.

### 5. **`runInterpreter` Function**

1. Loads initial CODE, EXEC, and INT stacks from the provided arrays.
2. While `EXEC` is non-empty:
   - Pop the top descriptor.
   - If `INSTRUCTION`, decode the opcode (e.g. `INTEGER_PLUS`) and do it (pop top 2 ints, push their sum).
   - If `INT_LITERAL`, push that integer onto the int stack.
   - If `SUBLIST`, parse that sublist into descriptors, then push those in reverse.

## How to Contribute

1. **Add New Instructions**  
   - Expand `OpCode` enum (e.g. `INTEGER_MINUS`, `INTEGER_MUL`, etc.).
   - Write a `makeInstruction` or `makeIntLiteral` function to build the descriptor for it.
   - In the main loop, implement the logic (pop the correct number of integers, do the operation, push the result).

2. **Support More Data Types**  
   - For instance, a `FLOAT` or `BOOLEAN` stack.  
   - Add a new tag (`FLOAT_LITERAL`?), new instructions (`FLOAT.PLUS`?), etc.

3. **Improve Parsing**  
   - If you want 256-bit literals, define a new token that reads 32 bytes, or do other expansions (like `CODE` type).  
   - Or define advanced sublist structures (like nested `( ( ) )` expansions).

4. **Gas Optimization**  
   - Check how large arrays are allocated (like in `parseSublist`).  
   - Possibly implement step-by-step execution if large code exceeds block gas limits.

5. **Add Logging & Debug Tools**  
   - Use `console.log(...)` to trace tokens, track final states, or measure performance.

6. **Test Coverage**  
   - Provide more unit tests in Foundry for advanced instructions, nested sublists, edge cases with zero-length sublists, etc.

## Next Steps

### A. Extending the VM

A “full” Push3 VM might include:
- **Multiple Data Stacks**: `BOOLEAN`, `FLOAT`, `NAME`, etc.  
- **NAME Binding**: Let `NAME`s reference values or code.  
- **Control Structures**: `EXEC.IF`, `EXEC.DO*COUNT`, or combinators like `EXEC.Y`.

### B. Example Project

You could use this interpreter in:
- A genetic programming environment, where random code is generated and mutated.
- A puzzle solver that runs tiny “scripts” in EVM-like constraints.

## Conclusion

This `Push3Interpreter` is a **minimal** scaffold for a token-based, list-expanding interpreter with typed descriptors. By following the patterns for instructions, sublists, and typed stacks, you can easily expand it into a more complete VM. We welcome contributions in the form of new instructions, data types, and performance optimizations.

1. **Clone** this repo.  
2. **Check** the `test/` folder for Foundry-based tests.  
3. **Add** or modify instructions & sublists in `parseSublist` and `runInterpreter`.  
4. **Run** `forge test --via-ir -vv` to see logs and confirm your changes.

Enjoy exploring the **Push3** style VM in Solidity!