#!/bin/bash
echo "=== Complete Evolution System Comparison ==="
echo ""

echo "1. ORIGINAL system (basic genetic operators)..."
cd /home/ubuntu/dev/SoLUSH-3/offchain
source $HOME/.cargo/env
echo "=== ORIGINAL SYSTEM RESULTS ===" > /tmp/complete_evolution_comparison.txt
timeout 120 cargo run --bin symreg_experiment 2>&1 | tee -a /tmp/complete_evolution_comparison.txt

echo ""
echo "2. IMPROVED system (enhanced genetic operators)..."
echo -e "\n=== IMPROVED SYSTEM RESULTS ===" >> /tmp/complete_evolution_comparison.txt
timeout 120 cargo run --bin symreg_improved 2>&1 | tee -a /tmp/complete_evolution_comparison.txt

echo ""
echo "3. ADVANCED system (population management)..."
echo -e "\n=== ADVANCED SYSTEM RESULTS ===" >> /tmp/complete_evolution_comparison.txt
timeout 120 cargo run --bin symreg_advanced 2>&1 | tee -a /tmp/complete_evolution_comparison.txt

echo ""
echo "4. EXPANDED system (expanded instruction set)..."
echo -e "\n=== EXPANDED SYSTEM RESULTS ===" >> /tmp/complete_evolution_comparison.txt
timeout 180 cargo run --bin symreg_expanded 2>&1 | tee -a /tmp/complete_evolution_comparison.txt

echo ""
echo "=== COMPARISON COMPLETE ==="
echo "All results saved to /tmp/complete_evolution_comparison.txt"
echo ""
echo "Quick summary by system:"
echo "Original (basic):"
grep "Best fitness" /tmp/complete_evolution_comparison.txt | head -3
echo ""
echo "Improved (genetic operators):"  
grep "Best fitness" /tmp/complete_evolution_comparison.txt | head -6 | tail -3
echo ""
echo "Advanced (population mgmt):"
grep "Best:" /tmp/complete_evolution_comparison.txt | head -3
echo ""
echo "Expanded (instruction set):"
grep "Best:" /tmp/complete_evolution_comparison.txt | tail -3