#!/usr/bin/env bash
# issue-observability-certs-from-ca.sh
#
# Issue TLS server certificates for Tempo, Prometheus, and Loki
# using the internal CA created by setup-internal-ca.sh.
#
# This script assumes:
#   - CA key/cert exist at /etc/ag-internal-ca/ca.key and /etc/ag-internal-ca/ca.crt
#   - Tempo, Prometheus, and Loki configs expect certs in their existing TLS dirs.
#
# It will:
#   1. Create CSRs and sign them with the internal CA
#   2. Place new certs/keys alongside existing ones, backing up old self-signed certs

set -euo pipefail

CA_DIR="/etc/ag-internal-ca"
CA_KEY="${CA_DIR}/ca.key"
CA_CERT="${CA_DIR}/ca.crt"
CA_DAYS=3650

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

if [[ "$EUID" -ne 0 ]]; then
  echo -e "${RED}This script must be run as root (use: sudo ./issue-observability-certs-from-ca.sh)${NC}"
  exit 1
fi

if [[ ! -f "${CA_KEY}" || ! -f "${CA_CERT}" ]]; then
  echo -e "${RED}CA key/cert not found in ${CA_DIR}. Run setup-internal-ca.sh first.${NC}"
  exit 1
fi

echo "═══════════════════════════════════════════════════════════"
echo "  Issue TLS Certs for Tempo, Prometheus, Loki from Internal CA"
echo "═══════════════════════════════════════════════════════════"

issue_cert() {
  local name="$1"       # tempo | prometheus | loki
  local key_path="$2"   # path to private key
  local crt_path="$3"   # path to cert
  local user="$4"       # service user (for chown), or "" for user services
  local san="$5"        # subjectAltName string

  echo ""
  echo "→ ${name^}: issuing certificate from internal CA"

  local dir
  dir="$(dirname "${crt_path}")"
  mkdir -p "${dir}"

  # Backup existing cert/key if present
  if [[ -f "${crt_path}" ]]; then
    cp "${crt_path}" "${crt_path}.backup-before-ca"
    echo -e "   ${YELLOW}• Backed up existing cert to ${crt_path}.backup-before-ca${NC}"
  fi
  if [[ -f "${key_path}" ]]; then
    cp "${key_path}" "${key_path}.backup-before-ca"
    echo -e "   ${YELLOW}• Backed up existing key to ${key_path}.backup-before-ca${NC}"
  fi

  local tmp_key tmp_csr tmp_ext
  tmp_key="${dir}/${name}.server.key.tmp"
  tmp_csr="${dir}/${name}.server.csr.tmp"
  tmp_ext="${dir}/${name}.server.ext.tmp"

  # Generate private key
  openssl genrsa -out "${tmp_key}" 2048

  # Create CSR
  openssl req -new -key "${tmp_key}" \
    -subj "/C=US/ST=State/L=City/O=AG-Internal/CN=${name}.local" \
    -out "${tmp_csr}"

  # Create extension file for SANs
  cat > "${tmp_ext}" <<EOF
basicConstraints=CA:FALSE
subjectAltName=${san}
keyUsage = digitalSignature, keyEncipherment
extendedKeyUsage = serverAuth
EOF

  # Sign with CA
  openssl x509 -req \
    -in "${tmp_csr}" \
    -CA "${CA_CERT}" \
    -CAkey "${CA_KEY}" \
    -CAcreateserial \
    -out "${crt_path}" \
    -days "${CA_DAYS}" \
    -sha256 \
    -extfile "${tmp_ext}"

  # Move key into place
  mv "${tmp_key}" "${key_path}"
  rm -f "${tmp_csr}" "${tmp_ext}"

  chmod 600 "${key_path}"
  chmod 644 "${crt_path}"
  if [[ -n "${user}" ]]; then
    chown "${user}:${user}" "${key_path}" "${crt_path}"
  fi

  echo -e "   ${GREEN}✓${NC} Issued new ${name} cert signed by internal CA"
  echo "   - Key:  ${key_path}"
  echo "   - Cert: ${crt_path}"
}

# Tempo (system service)
TEMPO_CERT_DIR="/etc/tempo/tls"
issue_cert "tempo" \
  "${TEMPO_CERT_DIR}/tempo.key" \
  "${TEMPO_CERT_DIR}/tempo.crt" \
  "tempo" \
  "DNS:localhost,DNS:tempo.local,IP:127.0.0.1"

# Prometheus (system service)
PROM_CERT_DIR="/etc/prometheus/tls"
issue_cert "prometheus" \
  "${PROM_CERT_DIR}/prometheus.key" \
  "${PROM_CERT_DIR}/prometheus.crt" \
  "prometheus" \
  "DNS:localhost,DNS:prometheus.local,IP:127.0.0.1"

# Loki (user service, files in $HOME)
# NOTE: We assume loki runs as your user and uses ~/.config/loki/tls
LOKI_CERT_DIR="${HOME:-/home/pde}/.config/loki/tls"
issue_cert "loki" \
  "${LOKI_CERT_DIR}/loki.key" \
  "${LOKI_CERT_DIR}/loki.crt" \
  "" \
  "DNS:localhost,DNS:loki.local,IP:127.0.0.1"


echo ""
echo "4. Restarting services to use new CA-signed certificates..."

systemctl restart tempo || echo -e "   ${YELLOW}• Tempo restart failed (check systemctl status tempo)${NC}"
systemctl restart prometheus || echo -e "   ${YELLOW}• Prometheus restart failed (check systemctl status prometheus)${NC}"
systemctl --user restart loki || echo -e "   ${YELLOW}• Loki restart failed (check systemctl --user status loki)${NC}"


echo ""
echo "═══════════════════════════════════════════════════════════"
echo -e "${GREEN}✅ Certificates issued from internal CA${NC}"
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "Next steps:"
echo "  1. In Grafana datasources, point to https://localhost:<port> for Tempo, Loki, Prometheus."
echo "  2. Turn OFF 'Skip TLS Verify' (certs should now validate via system trust)."
echo "  3. Remove *_INSECURE_TLS flags from AG backend and collectors where possible."
echo ""