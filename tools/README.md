# Development Tools

This directory contains useful scripts and tools for development and testing.

## Shell Scripts

### Evolution Testing  
- `run_evolution.sh` - Run full evolution experiment (3 min timeout)
- `run_short_evolution.sh` - Run short evolution test (1 min timeout)
- `run_original_evolution.sh` - Run original evolution system (5 min timeout)
- `run_improved_evolution.sh` - Run enhanced evolution system (5 min timeout)
- `compare_evolution.sh` - Compare original vs improved systems

### Debug Testing  
- `run_debug.sh` - Run debug execution tests
- `run_micro.sh` - Run micro-level operation tests
- `run_simple.sh` - Run simple manual tests
- `run_detailed.sh` - Run detailed debug analysis
- `run_direct.sh` - Run direct descriptor tests

## Usage

All scripts assume you're running from the project root:
```bash
./tools/run_evolution.sh
```

Or make them executable and run directly:
```bash
chmod +x tools/*.sh
tools/run_evolution.sh
```