#!/bin/bash
echo "=== Comprehensive Evolution System Comparison ==="
echo ""

echo "1. ORIGINAL system (basic genetic operators)..."
cd /home/ubuntu/dev/SoLUSH-3/offchain
source $HOME/.cargo/env
echo "=== ORIGINAL SYSTEM RESULTS ===" > /tmp/evolution_comparison.txt
timeout 120 cargo run --bin symreg_experiment 2>&1 | tee -a /tmp/evolution_comparison.txt

echo ""
echo "2. IMPROVED system (enhanced genetic operators)..."
echo -e "\n=== IMPROVED SYSTEM RESULTS ===" >> /tmp/evolution_comparison.txt
timeout 120 cargo run --bin symreg_improved 2>&1 | tee -a /tmp/evolution_comparison.txt

echo ""
echo "3. ADVANCED system (population management)..."
echo -e "\n=== ADVANCED SYSTEM RESULTS ===" >> /tmp/evolution_comparison.txt
timeout 120 cargo run --bin symreg_advanced 2>&1 | tee -a /tmp/evolution_comparison.txt

echo ""
echo "=== COMPARISON COMPLETE ==="
echo "All results saved to /tmp/evolution_comparison.txt"
echo ""
echo "Quick summary:"
echo "Original system:"
grep "Best fitness" /tmp/evolution_comparison.txt | head -5
echo ""
echo "Advanced system:"  
grep "Best:" /tmp/evolution_comparison.txt | head -5