# SoLUSH-3 Evolution Guide

This guide explains how to use the offchain Rust components to run genetic programming evolution experiments with the SoLUSH-3 Push3 virtual machine.

## Prerequisites

### 1. Environment Setup
```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Verify installation
cargo --version
```

### 2. Build Solidity Contracts
```bash
cd onchain
forge build --via-ir
```

**Note**: The `--via-ir` flag is required to handle stack depth in the Push3 interpreter contract.

### 3. Verify Contract Artifacts
Ensure the following file exists:
```
onchain/out/Push3Interpreter.sol/Push3Interpreter.json
```

## Available Evolution Commands

### 1. Basic GP Tester (`gp_tester`)
Tests basic genetic programming operations on small programs.

```bash
cd offchain
cargo run --bin gp_tester
```

**Purpose**: 
- Generates 3 random AST programs
- Tests mutation operations
- Verifies EVM integration works
- Good for debugging basic functionality

**Expected Output**:
```
=== Program 0 ===
Random AST:
Instruction(Plus)
Original AST ran successfully!
Final INT stack: [...]
```

### 2. Symbolic Regression Experiment (`symreg_experiment`)
Main evolution experiment to evolve the function `f(x) = 3x² + x + 3`.

```bash
cd offchain
cargo run --bin symreg_experiment
```

**Parameters**:
- Population size: 4000 individuals
- Generations: 100
- Target function: `f(x) = 3x² + x + 3`
- Test inputs: x ∈ [-5, 5]
- Early stopping: Error < 1.0

**Expected Output**:
```
=== Generation 0 ===
Best error = 1801

=== Generation 1 ===
Best error = 1650

=== Generation 2 ===
Best error = 1420
...
```

### 3. GP Specification Tester (`gp_spec_tester`)
Tests genetic programming operations against Push3 specification compliance.

```bash
cd offchain
cargo run --bin gp_spec_tester
```

### 4. Local Symbolic Regression (`symreg_experiment_local`)
Smaller-scale testing version of the symbolic regression experiment.

```bash
cd offchain
cargo run --bin symreg_experiment_local
```

## Understanding Evolution Output

### Generation Reports
Each generation shows:
```
=== Generation N ===
Best error = X.X
```

- **Generation N**: Current evolutionary generation
- **Best error**: Mean squared error of the best individual
- **Lower error = better fitness**

### Final Results
At the end, the top 10 evolved programs are displayed:
```
=== Final Population (Top 10) ===
Subject #0, err=42.5, AST:
Sublist([
    IntLiteral(3),
    Instruction(Mult),
    IntLiteral(1),
    Instruction(Plus),
    ...
])
```

## Evolution Algorithm Details

### Genetic Operations

1. **Population Initialization**: Random AST generation
2. **Selection**: Tournament selection based on fitness
3. **Reproduction Strategy** (per generation):
   - 25% elite individuals (best performers)
   - 25% random new individuals
   - 25% crossover offspring
   - 25% mutated individuals

### Fitness Evaluation
Programs are evaluated on inputs x ∈ [-5, 5] against target `f(x) = 3x² + x + 3`:
```rust
// Gradual fitness function with partial rewards:
fitness = Σ reward_for_sample(predicted(x), target(x)) / number_of_samples

where reward_for_sample provides:
- 1000 points for exact match
- 100-50 points for close values (diff ≤ 10)  
- 20-10 points for reasonable values (diff ≤ 100)
- 0.1 points for execution failures
- 20% bonus for programs that execute on all samples
```

### Available Operations
Current instruction set includes:
- **Arithmetic**: `+`, `-`, `*`
- **Stack**: `DUP`, `POP`
- **Literals**: Integer constants
- **Ephemeral Random Constants**: Random values generated during evolution

## Configuration Parameters

Key parameters can be modified in the source code:

### Population Parameters (`symreg_experiment.rs`)
```rust
let pop_size = 4000;              // Population size
let generations = 100;            // Maximum generations
let early_stop_threshold = 1.0;   // Stop if error < threshold
let max_points = 20;              // Maximum program size
```

### Instruction Set (`generate_spec.rs`)
```rust
atoms: vec![
    Opcode(Plus),
    Opcode(Minus),
    Opcode(Mult),
    Opcode(Dup),
    EphemeralInt,  // Random constants
]
```

## Troubleshooting

### Common Issues

#### 1. "Call reverted" Errors
```
Original AST error: Call reverted: gas used=22564, output=0x
```
**Cause**: Opcode mapping mismatch between Rust and Solidity
**Solution**: Verify opcode mappings in `offchain/src/compiler/ast.rs`

#### 2. Contract Not Found
```
Failed to read JSON file ../onchain/out/Push3Interpreter.sol/Push3Interpreter.json
```
**Solution**: Build contracts first with `forge build --via-ir`

#### 3. No Evolution Progress
If fitness stays constant across generations:
- Check if programs are executing (no revert errors)
- Verify fitness function is working
- Increase population diversity
- Adjust mutation rates
- Check that fitness values are reasonable (should be 1-1000+ range)

### Debug Mode
For detailed debugging, examine individual programs:
```bash
# Run with smaller population and more verbose output
# Modify source code to print AST details
```

## Extending the System

### Adding New Operations
1. Add to `OpCode` enum in `offchain/src/compiler/ast.rs`
2. Update opcode mapping in `DefaultOpCodeMapping`
3. Implement operation in Solidity interpreter
4. Add to instruction set in `generate_spec.rs`

### Modifying Fitness Functions
Edit the `evaluate_fitness` function in experiment files to:
- Change target functions
- Adjust error calculations
- Add parsimony pressure
- Include multiple test cases

### Custom Evolution Strategies
Modify reproduction logic in experiment files:
- Change selection methods
- Adjust crossover/mutation ratios
- Implement elitism strategies
- Add diversity maintenance

## Performance Notes

- **Compilation time**: First run takes ~2 minutes for dependency compilation
- **Execution time**: 100 generations with 4000 individuals ≈ 10-30 minutes
- **Memory usage**: Large populations require significant RAM
- **Disk space**: Contract artifacts require ~100MB

## Success Indicators

### Working System Signs:
✅ Programs execute without "Call reverted" errors  
✅ Fitness values improve between generations (higher = better)  
✅ Evolution completes without crashes  
✅ Final population shows diverse programs  
✅ Best fitness values reach 100+ (good programs) or 200+ (excellent programs)

### System Issues Signs:
❌ All programs revert during execution  
❌ Fitness stays exactly constant across generations  
❌ Rust compilation failures  
❌ Contract deployment failures  
❌ All fitness values below 10 (indicates execution failures)

## Next Steps

Priority improvements for enhanced evolution performance:
1. ~~**Better fitness function design**~~ ✅ **COMPLETED** - Gradual reward system implemented
2. **More diverse genetic operators** - Point mutation, better crossover strategies, size-aware operations
3. **Population management improvements** - Enhanced elitism, diversity maintenance, niching
4. **Expanded instruction set** - Comparisons, constants, control flow, conditionals

Additional enhancements:
5. **Add problem domains** - Control problems, symbolic regression variants, multi-objective optimization
6. **Optimize performance** - Parallel evaluation, smarter selection, caching
7. **Advanced features** - Automatically defined functions (ADFs), co-evolution, novelty search

## Recent Fixes and Improvements

### Version 2024-01 - Evolution System Restored
- **Fixed critical SUBLIST tag bug**: Corrected tag value from 3 to 4 to match Solidity contract
- **Improved fitness function**: Replaced harsh MSE with gradual reward system for better evolution
- **Fixed ABI mismatch**: Added missing `bool[]` parameter to interpreter calls
- **Verified execution**: All basic operations (literals, arithmetic, stack ops) now work correctly

### Expected Results After Fixes
With the current system, you should see:
- **Generation 0**: Initial fitness ~50-120
- **Early generations**: Gradual improvements to ~120-150  
- **Later generations**: Potential breakthroughs to 200+ fitness
- **Best programs**: Can achieve 200+ fitness with complex nested operations

### Example Successful Evolution
```
=== Generation 0 ===
Best fitness = 118.59574455567366

=== Generation 2 ===  
Best fitness = 119.1949509820323

=== Generation 4 ===
Best fitness = 224.95606910371487
```

The system now successfully evolves working Push3 programs that can approximate the target function `f(x) = 3x² + x + 3`.

For questions or issues, refer to the main project documentation or examine the source code in the `offchain/src/` directory.