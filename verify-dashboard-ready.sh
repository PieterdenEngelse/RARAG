#!/bin/bash
# Verification script for Grafana dashboard setup

echo "🔍 Checking Multi-Source Logging Setup..."
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check Vector
echo -n "1. Vector service: "
if systemctl --user is-active --quiet vector; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${RED}✗ Not running${NC}"
    echo "   Fix: systemctl --user start vector"
fi

# Check Loki
echo -n "2. Loki service: "
if systemctl --user is-active --quiet loki 2>/dev/null || curl -s http://127.0.0.1:3100/ready > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${RED}✗ Not running${NC}"
    echo "   Fix: systemctl --user start loki"
fi

# Check Grafana
echo -n "3. Grafana service: "
if curl -s http://127.0.0.1:3000/api/health > /dev/null 2>&1; then
    echo -e "${GREEN}✓ Running${NC}"
else
    echo -e "${RED}✗ Not running${NC}"
    echo "   Fix: sudo systemctl start grafana-server"
fi

echo ""
echo "📊 Checking Log Sources..."
echo ""

# Query Loki for available jobs
JOBS=$(curl -s http://127.0.0.1:3100/loki/api/v1/label/job/values 2>/dev/null | grep -o '"[^"]*"' | grep -v "status\|success\|data" | tr -d '"')

if [ -z "$JOBS" ]; then
    echo -e "${RED}✗ Cannot query Loki${NC}"
    exit 1
fi

# Check each required source
for source in "systemd-journal" "system-errors" "kernel" "auth"; do
    echo -n "   - $source: "
    if echo "$JOBS" | grep -q "$source"; then
        echo -e "${GREEN}✓ Available${NC}"
    else
        echo -e "${YELLOW}⚠ Not found${NC}"
    fi
done

echo ""
echo "🏷️  Available systemd units:"
UNITS=$(curl -s http://127.0.0.1:3100/loki/api/v1/label/systemd_unit/values 2>/dev/null | grep -o '"[^"]*"' | grep -v "status\|success\|data\|null" | tr -d '"')
for unit in $UNITS; do
    echo "   - $unit"
done

echo ""
echo "📈 Sample Log Count (last 5 minutes):"
echo ""

# Count logs from each source
for source in "systemd-journal" "system-errors" "kernel" "auth"; do
    COUNT=$(curl -s -G http://127.0.0.1:3100/loki/api/v1/query \
        --data-urlencode "query=count_over_time({job=\"$source\"}[5m])" \
        2>/dev/null | grep -o '"value":\[[^]]*\]' | grep -o '[0-9]*' | tail -1)
    
    if [ -n "$COUNT" ] && [ "$COUNT" -gt 0 ]; then
        echo -e "   $source: ${GREEN}$COUNT logs${NC}"
    else
        echo -e "   $source: ${YELLOW}0 logs${NC}"
    fi
done

echo ""
echo "📁 Dashboard Files:"
echo ""

if [ -f "./grafana-multi-source-logs-dashboard.json" ]; then
    echo -e "   ${GREEN}✓${NC} grafana-multi-source-logs-dashboard.json"
else
    echo -e "   ${RED}✗${NC} grafana-multi-source-logs-dashboard.json (missing)"
fi

if [ -f "./GRAFANA_DASHBOARD_IMPORT_GUIDE.md" ]; then
    echo -e "   ${GREEN}✓${NC} GRAFANA_DASHBOARD_IMPORT_GUIDE.md"
else
    echo -e "   ${RED}✗${NC} GRAFANA_DASHBOARD_IMPORT_GUIDE.md (missing)"
fi

echo ""
echo "🚀 Next Steps:"
echo ""
echo "1. Open Grafana: http://localhost:3000"
echo "2. Go to Dashboards → Import"
echo "3. Upload: grafana-multi-source-logs-dashboard.json"
echo "4. Select Loki as data source"
echo "5. Click Import"
echo ""
echo "📖 Full guide: cat GRAFANA_DASHBOARD_IMPORT_GUIDE.md"
echo ""
