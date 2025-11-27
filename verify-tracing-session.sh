#!/bin/bash
# Verification script for tracing session changes
# Tests TLS fix and resource attribution implementation

set -e

echo "═══════════════════════════════════════════════════════════"
echo "  Tracing Session Verification Script"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
AG_URL="http://127.0.0.1:3010"
METRICS_URL="${AG_URL}/monitoring/metrics"
HEALTH_URL="${AG_URL}/monitoring/health"

# Function to check if AG is running
check_ag_running() {
    echo -n "Checking if AG backend is running... "
    if curl -s -f "${HEALTH_URL}" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Running${NC}"
        return 0
    else
        echo -e "${RED}✗ Not running${NC}"
        echo ""
        echo "Please start the AG backend first:"
        echo "  cargo run --release"
        echo "  # or"
        echo "  ./target/release/ag"
        exit 1
    fi
}

# Function to check trace alerting
check_trace_alerting() {
    echo ""
    echo "─────────────────────────────────────────────────────────"
    echo "Trace Alerting Verification"
    echo "─────────────────────────────────────────────────────────"
    
    echo -n "Checking trace_alert_checks_total metric... "
    METRICS=$(curl -s "${METRICS_URL}")
    
    if echo "$METRICS" | grep -q "trace_alert_checks_total"; then
        echo -e "${GREEN}✓ Found${NC}"
        
        # Check for successful checks
        echo -n "Checking for successful trace checks (status=\"ok\")... "
        if echo "$METRICS" | grep "trace_alert_checks_total" | grep -q 'status="ok"'; then
            OK_COUNT=$(echo "$METRICS" | grep 'trace_alert_checks_total{status="ok"}' | awk '{print $2}')
            echo -e "${GREEN}✓ Found (count: ${OK_COUNT})${NC}"
            
            if [ "$OK_COUNT" -gt 0 ]; then
                echo -e "${GREEN}  → TLS fix is working!${NC}"
            else
                echo -e "${YELLOW}  → Wait 30 seconds for first check${NC}"
            fi
        else
            echo -e "${YELLOW}⚠ Not found yet${NC}"
            echo "  Wait 30 seconds for first trace check"
        fi
        
        # Check for errors
        echo -n "Checking for trace check errors (status=\"error\")... "
        if echo "$METRICS" | grep "trace_alert_checks_total" | grep -q 'status="error"'; then
            ERROR_COUNT=$(echo "$METRICS" | grep 'trace_alert_checks_total{status="error"}' | awk '{print $2}')
            if [ "$ERROR_COUNT" -gt 0 ]; then
                echo -e "${YELLOW}⚠ Found (count: ${ERROR_COUNT})${NC}"
                echo "  This may be from before the restart"
            else
                echo -e "${GREEN}✓ None${NC}"
            fi
        else
            echo -e "${GREEN}✓ None${NC}"
        fi
        
        # Check for anomalies
        echo -n "Checking for trace anomalies... "
        if echo "$METRICS" | grep -q "trace_anomalies_total"; then
            echo -e "${GREEN}✓ Metric exists${NC}"
            
            # Show anomaly counts by type
            for TYPE in high_latency error_status high_error_rate; do
                if echo "$METRICS" | grep "trace_anomalies_total" | grep -q "type=\"${TYPE}\""; then
                    COUNT=$(echo "$METRICS" | grep "trace_anomalies_total{type=\"${TYPE}\"}" | awk '{print $2}')
                    if [ "$COUNT" -gt 0 ]; then
                        echo "  - ${TYPE}: ${COUNT}"
                    fi
                fi
            done
        else
            echo -e "${YELLOW}⚠ Not found${NC}"
        fi
    else
        echo -e "${RED}✗ Not found${NC}"
        echo "  Trace alerting may not be enabled"
    fi
}

# Function to check resource attribution
check_resource_attribution() {
    echo ""
    echo "─────────────────────────────────────────────────────────"
    echo "Resource Attribution Verification"
    echo "─────────────────────────────────────────────────────────"
    
    METRICS=$(curl -s "${METRICS_URL}")
    
    # Check each metric
    METRICS_TO_CHECK=(
        "process_memory_bytes"
        "process_memory_peak_bytes"
        "process_cpu_percent"
        "tracing_memory_overhead_bytes"
        "tracing_cpu_overhead_percent"
    )
    
    ALL_FOUND=true
    for METRIC in "${METRICS_TO_CHECK[@]}"; do
        echo -n "Checking ${METRIC}... "
        if echo "$METRICS" | grep -q "^${METRIC} "; then
            VALUE=$(echo "$METRICS" | grep "^${METRIC} " | awk '{print $2}')
            echo -e "${GREEN}✓ ${VALUE}${NC}"
        else
            echo -e "${RED}✗ Not found${NC}"
            ALL_FOUND=false
        fi
    done
    
    if [ "$ALL_FOUND" = true ]; then
        echo ""
        echo -e "${GREEN}✓ All resource attribution metrics found!${NC}"
        
        # Calculate and display overhead percentage
        MEM_TOTAL=$(echo "$METRICS" | grep "^process_memory_bytes " | awk '{print $2}')
        MEM_OVERHEAD=$(echo "$METRICS" | grep "^tracing_memory_overhead_bytes " | awk '{print $2}')
        
        if [ -n "$MEM_TOTAL" ] && [ -n "$MEM_OVERHEAD" ] && [ "$MEM_TOTAL" -gt 0 ]; then
            OVERHEAD_PCT=$(echo "scale=2; ($MEM_OVERHEAD / $MEM_TOTAL) * 100" | bc)
            echo ""
            echo "Resource Overhead Summary:"
            echo "  Memory Total: $(numfmt --to=iec-i --suffix=B $MEM_TOTAL)"
            echo "  Memory Overhead: $(numfmt --to=iec-i --suffix=B $MEM_OVERHEAD) (${OVERHEAD_PCT}%)"
            
            CPU_OVERHEAD=$(echo "$METRICS" | grep "^tracing_cpu_overhead_percent " | awk '{print $2}')
            echo "  CPU Overhead: ${CPU_OVERHEAD}%"
        fi
    else
        echo ""
        echo -e "${YELLOW}⚠ Some metrics missing${NC}"
        echo "  Resource attribution may not be active yet"
        echo "  Wait 60 seconds for first update"
    fi
}

# Function to check Tempo connectivity
check_tempo() {
    echo ""
    echo "─────────────────────────────────────────────────────────"
    echo "Tempo Connectivity Verification"
    echo "─────────────────────────────────────────────────────────"
    
    echo -n "Checking Tempo HTTPS endpoint (https://localhost:3200)... "
    if curl -s -k -f "https://localhost:3200/api/search?start=0&end=9999999999&limit=1" > /dev/null 2>&1; then
        echo -e "${GREEN}✓ Accessible${NC}"
    else
        echo -e "${YELLOW}⚠ Not accessible${NC}"
        echo "  This is expected if Tempo is not running"
    fi
    
    echo -n "Checking Tempo service status... "
    if systemctl is-active --quiet tempo.service 2>/dev/null; then
        echo -e "${GREEN}✓ Running${NC}"
    else
        echo -e "${YELLOW}⚠ Not running or not accessible${NC}"
    fi
}

# Function to show recent logs
show_logs() {
    echo ""
    echo "─────────────────────────────────────────────────────────"
    echo "Recent AG Backend Logs (last 20 lines)"
    echo "─────────────────────────────────────────────────────────"
    
    # Try to find AG process and show its output
    AG_PID=$(pgrep -f "target/release/ag" | head -1)
    
    if [ -n "$AG_PID" ]; then
        echo "AG Backend PID: $AG_PID"
        echo ""
        echo "Look for these startup messages:"
        echo "  - 🔔 Trace-based alerting started"
        echo "  - 📊 Resource attribution started"
        echo ""
        echo "If running in background, check logs with:"
        echo "  journalctl --user -u ag.service -n 20"
        echo "  # or"
        echo "  tail -f logs/ag.log"
    else
        echo "AG backend process not found"
    fi
}

# Main execution
main() {
    check_ag_running
    check_trace_alerting
    check_resource_attribution
    check_tempo
    show_logs
    
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "  Verification Complete"
    echo "═══════════════════════════════════════════════════════════"
    echo ""
    echo "Next steps:"
    echo "  1. If metrics are missing, wait 60 seconds and run again"
    echo "  2. Check Grafana for trace visualization"
    echo "  3. Monitor metrics over time for trends"
    echo ""
    echo "For detailed metrics, run:"
    echo "  curl ${METRICS_URL} | grep -E '(trace_|process_|tracing_)'"
    echo ""
}

# Run main function
main
