#!/usr/bin/env bash
# setup-internal-ca.sh
#
# Create a small internal Certificate Authority (CA) for local observability stack
# (Tempo, Prometheus, Loki, AG backend, Grafana).
#
# This script:
#   1. Creates /etc/ag-internal-ca
#   2. Generates a CA private key and root certificate (10-year validity)
#   3. Optionally installs the CA cert into the OS trust store (Debian/Ubuntu-style)
#
# NOTE:
#   - Run this once as a privileged user: sudo ./setup-internal-ca.sh
#   - It is SAFE to re-run; it will refuse to overwrite an existing CA.

set -euo pipefail

CA_DIR="/etc/ag-internal-ca"
CA_KEY="${CA_DIR}/ca.key"
CA_CERT="${CA_DIR}/ca.crt"
CA_DAYS=3650   # 10 years

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

if [[ "$EUID" -ne 0 ]]; then
  echo -e "${RED}This script must be run as root (use: sudo ./setup-internal-ca.sh)${NC}"
  exit 1
fi

echo "═══════════════════════════════════════════════════════════"
echo "  AG Internal CA Setup"
echo "═══════════════════════════════════════════════════════════"

# 1. Create CA directory
if [[ ! -d "${CA_DIR}" ]]; then
  echo "1. Creating CA directory at ${CA_DIR} ..."
  mkdir -p "${CA_DIR}"
  chmod 700 "${CA_DIR}"
  echo -e "   ${GREEN}✓${NC} Directory created"
else
  echo "1. CA directory already exists: ${CA_DIR}"
fi

# 2. Generate CA key & cert if not present
if [[ -f "${CA_KEY}" || -f "${CA_CERT}" ]]; then
  echo "2. CA key or cert already exists:"
  echo "   - Key:  ${CA_KEY}"
  echo "   - Cert: ${CA_CERT}"
  echo -e "   ${YELLOW}Skipping CA generation to avoid overwriting existing CA.${NC}"
else
  echo "2. Generating CA private key and root certificate ..."

  openssl genrsa -out "${CA_KEY}" 4096

  openssl req -x509 -new -nodes \
    -key "${CA_KEY}" \
    -sha256 \
    -days "${CA_DAYS}" \
    -out "${CA_CERT}" \
    -subj "/C=US/ST=State/L=City/O=AG-Internal/CN=ag-internal-ca"

  chmod 600 "${CA_KEY}"
  chmod 644 "${CA_CERT}"

  echo -e "   ${GREEN}✓${NC} CA created"
  echo "   - CA Key:  ${CA_KEY}"
  echo "   - CA Cert: ${CA_CERT}"
  echo "   - Validity: ${CA_DAYS} days (~10 years)"
fi

# 3. Optionally install CA into system trust store (Debian/Ubuntu style)
TRUST_DIR="/usr/local/share/ca-certificates"
TRUST_TARGET="${TRUST_DIR}/ag-internal-ca.crt"

if [[ -d "${TRUST_DIR}" ]]; then
  echo ""
  echo "3. Installing CA certificate into system trust store (Debian/Ubuntu)..."
  cp "${CA_CERT}" "${TRUST_TARGET}"
  chmod 644 "${TRUST_TARGET}"
  echo "   Running update-ca-certificates ..."
  update-ca-certificates >/dev/null 2>&1 || true
  echo -e "   ${GREEN}✓${NC} CA installed to system trust (if supported by this distro)"
  echo "   - Trusted CA: ${TRUST_TARGET}"
else
  echo ""
  echo -e "3. ${YELLOW}System trust directory not found at ${TRUST_DIR}.${NC}"
  echo "   Please install the CA manually into your OS trust store or browser if needed."
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo -e "${GREEN}✅ Internal CA setup complete${NC}"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Next steps:"
echo "  1. Use this CA to sign TLS certificates for Tempo, Prometheus, and Loki."
echo "  2. Then disable all 'Skip TLS' / 'insecure' flags in Grafana and AG backend."
echo ""
echo "Run: ./issue-observability-certs-from-ca.sh to re-issue service certificates."
echo ""