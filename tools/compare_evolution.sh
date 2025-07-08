#!/bin/bash
echo "=== Comparing Evolution Systems ==="
echo ""

echo "Running ORIGINAL evolution system..."
cd /home/ubuntu/dev/SoLUSH-3/offchain
source $HOME/.cargo/env
echo "Original system results:" > /tmp/original_results.txt
timeout 180 cargo run --bin symreg_experiment 2>&1 | tee -a /tmp/original_results.txt

echo ""
echo "Running IMPROVED evolution system..."
echo "Improved system results:" > /tmp/improved_results.txt  
timeout 180 cargo run --bin symreg_improved 2>&1 | tee -a /tmp/improved_results.txt

echo ""
echo "=== COMPARISON COMPLETE ==="
echo "Results saved to /tmp/original_results.txt and /tmp/improved_results.txt"