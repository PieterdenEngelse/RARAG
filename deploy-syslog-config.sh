#!/bin/bash
# Deploy Vector configuration with syslog support

set -e

echo "🔧 Deploying Vector configuration with syslog support..."
echo ""

# Validate the new config
echo "1. Validating configuration..."
if vector validate ./vector-with-syslog.toml; then
    echo "   ✅ Configuration is valid"
else
    echo "   ❌ Configuration validation failed!"
    exit 1
fi

echo ""
echo "2. Backing up current configuration..."
cp /home/pde/.config/vector/vector.toml /home/pde/.config/vector/vector.toml.backup-syslog
echo "   ✅ Backup created"

echo ""
echo "3. Deploying new configuration..."
cp ./vector-with-syslog.toml /home/pde/.config/vector/vector.toml
echo "   ✅ Configuration deployed"

echo ""
echo "4. Restarting Vector service..."
systemctl --user restart vector
sleep 2

echo ""
echo "5. Checking Vector status..."
if systemctl --user is-active --quiet vector; then
    echo "   ✅ Vector is running"
else
    echo "   ❌ Vector failed to start!"
    echo ""
    echo "Checking logs:"
    journalctl --user -u vector -n 20 --no-pager
    exit 1
fi

echo ""
echo "6. Waiting for syslog data..."
sleep 5

echo ""
echo "7. Verifying syslog job in Loki..."
JOBS=$(curl -s http://127.0.0.1:3100/loki/api/v1/label/job/values | jq -r '.data[]' 2>/dev/null)

if echo "$JOBS" | grep -q "syslog"; then
    echo "   ✅ Syslog job found in Loki!"
else
    echo "   ⚠️  Syslog job not yet visible (may take a moment)"
fi

echo ""
echo "📊 Available log sources:"
echo "$JOBS" | sed 's/^/   - /'

echo ""
echo "✅ Deployment complete!"
echo ""
echo "🎯 Next steps:"
echo "1. Wait 1-2 minutes for syslog data to appear"
echo "2. Query: {job=\"syslog\"}"
echo "3. Query errors only: {job=\"syslog\", is_error=\"true\"}"
echo ""
