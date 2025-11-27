#!/bin/bash
# Master script to complete distributed tracing setup

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  AG Backend Distributed Tracing - Final Setup            ║"
echo "╚═══════════════════════════════════════════════════════════╝"
echo ""
echo "This script will complete the distributed tracing setup by:"
echo "  1. Updating Tempo configuration (requires sudo)"
echo "  2. Restarting Tempo service"
echo "  3. Restarting OpenTelemetry Collector"
echo "  4. Verifying traces are flowing"
echo ""
read -p "Press Enter to continue or Ctrl+C to cancel..."
echo ""

# Step 1: Update Tempo configuration
echo "═══════════════════════════════════════════════════════════"
echo "Step 1: Update Tempo Configuration"
echo "═══════════════════════════════════════════════════════════"
echo ""
bash /home/pde/ag/update-tempo-config.sh
if [ $? -ne 0 ]; then
    echo "❌ Failed to update Tempo configuration"
    exit 1
fi
echo ""

# Step 2: Update OpenTelemetry Collector
echo "═══════════════════════════════════════════════════════════"
echo "Step 2: Restart OpenTelemetry Collector"
echo "═══════════════════════════════════════════════════════════"
echo ""
bash /home/pde/ag/update-otelcol-config.sh
if [ $? -ne 0 ]; then
    echo "❌ Failed to restart OpenTelemetry Collector"
    exit 1
fi
echo ""

# Step 3: Final verification
echo "═══════════════════════════════════════════════════════════"
echo "Step 3: Final Verification"
echo "═══════════════════════════════════════════════════════════"
echo ""
bash /home/pde/ag/final-verification.sh
echo ""

echo "╔═══════════════════════════════════════════════════════════╗"
echo "║  Setup Complete!                                          ║"
echo "╚═══════════════════════════════════════════════════════════╝"
