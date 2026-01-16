# Financial Pipeline - Trading Integration Security Specification

**Version:** 1.0
**Classification:** INTERNAL - SECURITY SENSITIVE
**Author:** KALIC
**Date:** 2026-01-16

---

## Table of Contents
1. [Architecture Overview](#architecture-overview)
2. [Threat Model](#threat-model)
3. [Security Requirements](#security-requirements)
4. [Implementation Phases](#implementation-phases)
5. [Kali Linux Security Audit Checklist](#kali-linux-security-audit-checklist)
6. [API Security](#api-security)
7. [Cryptographic Requirements](#cryptographic-requirements)
8. [Network Security](#network-security)
9. [Incident Response](#incident-response)

---

## Architecture Overview

### Trading Integration Options

```
┌─────────────────────────────────────────────────────────────────┐
│                    FINANCIAL PIPELINE                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │   UI Layer   │───▶│  Rust Core   │───▶│   SQLite     │       │
│  │   (Tauri)    │    │  (Business)  │    │   (Local)    │       │
│  └──────────────┘    └──────┬───────┘    └──────────────┘       │
│                             │                                    │
│         ┌───────────────────┼───────────────────┐               │
│         ▼                   ▼                   ▼               │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │   Alpaca     │    │  DEX/Web3    │    │    IBKR      │       │
│  │  (Stocks)    │    │  (Crypto)    │    │  (Global)    │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow - Order Execution

```
User Input ─▶ Input Validation ─▶ Business Logic ─▶ Risk Check
                                                        │
                                                        ▼
Audit Log ◀── Order Confirmation ◀── API Response ◀── Broker API
```

---

## Threat Model

### STRIDE Analysis

| Threat | Category | Risk | Mitigation |
|--------|----------|------|------------|
| API key theft | Spoofing | CRITICAL | HSM/Keyring storage, never in code |
| Order tampering | Tampering | HIGH | Request signing, TLS 1.3 only |
| Trade repudiation | Repudiation | MEDIUM | Immutable audit logs |
| Credential exposure | Info Disclosure | CRITICAL | Memory-safe handling, zeroing |
| Unauthorized trading | Elevation | CRITICAL | Multi-factor confirmation |
| API flooding | DoS | MEDIUM | Rate limiting, circuit breakers |

### Attack Vectors

```
EXTERNAL THREATS:
├── Network interception (MITM)
├── API key extraction from memory
├── Malicious dependencies (supply chain)
├── DNS hijacking to fake endpoints
├── Replay attacks on signed requests
└── Social engineering for credentials

INTERNAL THREATS:
├── Malware on local machine
├── Debug logs exposing secrets
├── Crash dumps containing keys
├── Browser extension access to WebView
└── Clipboard monitoring
```

### Assets to Protect

| Asset | Classification | Protection Level |
|-------|---------------|------------------|
| API Keys | SECRET | Encrypted at rest, HSM preferred |
| Private Keys (Crypto) | SECRET | Hardware wallet or encrypted keystore |
| Order History | CONFIDENTIAL | Encrypted SQLite, local only |
| Account Balance | CONFIDENTIAL | Never cached, always fresh API call |
| User Credentials | SECRET | Never stored, OAuth preferred |

---

## Security Requirements

### SR-001: Credential Storage
```
MUST: Use OS keyring/keychain (Windows Credential Manager, macOS Keychain)
MUST: Never store credentials in:
  - Source code
  - Config files
  - Environment variables (in production)
  - SQLite database (unencrypted)
  - Log files

SHOULD: Support hardware security keys (YubiKey)
SHOULD: Implement credential rotation reminders
```

### SR-002: API Communication
```
MUST: TLS 1.3 only, reject downgrade
MUST: Certificate pinning for known broker endpoints
MUST: Request signing with timestamp (prevent replay)
MUST: Timeout all requests (30s max)

MUST NOT: Trust self-signed certificates
MUST NOT: Disable certificate validation
MUST NOT: Log request/response bodies containing secrets
```

### SR-003: Order Validation
```
MUST: Validate all inputs server-side equivalent
MUST: Implement order size limits (configurable)
MUST: Require confirmation for orders > threshold
MUST: Rate limit order submissions (e.g., 10/minute)

CHECKS:
  - Symbol exists and is tradeable
  - Quantity > 0 and within limits
  - Price within reasonable range (±50% of last)
  - Sufficient balance/buying power
  - Market hours check (for stocks)
```

### SR-004: Audit Logging
```
MUST: Log all order attempts (success and failure)
MUST: Include timestamp, action, parameters, result
MUST: Logs append-only, tamper-evident
MUST: Retain logs minimum 7 years (regulatory)

MUST NOT: Log credentials, full card numbers, private keys
```

### SR-005: Memory Safety
```
MUST: Zero sensitive data after use
MUST: Use Rust's secure memory handling
MUST: Disable core dumps in production
MUST: Clear clipboard after paste operations

RUST PATTERNS:
  - Use `secrecy` crate for secrets
  - Use `zeroize` crate for cleanup
  - Avoid String for secrets (use SecretString)
```

---

## Implementation Phases

### Phase 1: Paper Trading (Week 1-2)
```
Goal: Full integration with NO real money

Tasks:
[ ] Alpaca paper trading API integration
[ ] Order placement UI
[ ] Position tracking
[ ] P&L calculation
[ ] Paper portfolio in SQLite

Security:
[ ] API key storage in OS keyring
[ ] Basic input validation
[ ] Request logging (non-sensitive)
```

### Phase 2: Security Hardening (Week 3-4)
```
Goal: Production-ready security

Tasks:
[ ] Certificate pinning implementation
[ ] Request signing
[ ] Rate limiting
[ ] Order confirmation dialogs
[ ] Audit logging system

Security Audit:
[ ] Kali Linux penetration testing
[ ] Dependency vulnerability scan
[ ] Memory leak/exposure testing
[ ] API fuzzing
```

### Phase 3: Live Trading (Week 5-6)
```
Goal: Real money capability

Tasks:
[ ] Live API key management
[ ] Multi-factor order confirmation
[ ] Emergency kill switch
[ ] Balance/position sync
[ ] Error handling & recovery

Security:
[ ] Final penetration test
[ ] Code audit
[ ] Incident response testing
```

### Phase 4: Crypto Integration (Week 7-8)
```
Goal: DEX trading capability

Tasks:
[ ] Wallet connection (WalletConnect/MetaMask)
[ ] DEX aggregator integration (1inch, Jupiter)
[ ] Gas estimation
[ ] Transaction signing
[ ] Confirmation tracking

Security:
[ ] Smart contract interaction audit
[ ] Phishing protection (URL verification)
[ ] Transaction simulation before execution
```

---

## Kali Linux Security Audit Checklist

### Pre-Audit Setup
```bash
# Update Kali tools
sudo apt update && sudo apt full-upgrade -y

# Install additional tools
sudo apt install -y \
  burpsuite \
  sqlmap \
  nikto \
  wfuzz \
  nuclei \
  ffuf \
  testssl.sh \
  rustscan
```

### 1. Network Security Testing

#### 1.1 TLS/SSL Analysis
```bash
# Test SSL configuration
testssl.sh --severity HIGH https://api.alpaca.markets

# Check for weak ciphers
nmap --script ssl-enum-ciphers -p 443 api.alpaca.markets

# Verify certificate chain
openssl s_client -connect api.alpaca.markets:443 -servername api.alpaca.markets

PASS CRITERIA:
[ ] TLS 1.3 supported
[ ] TLS 1.0/1.1 disabled
[ ] No weak ciphers (RC4, DES, MD5)
[ ] Valid certificate chain
[ ] HSTS header present
```

#### 1.2 Network Interception Test
```bash
# Set up MITM proxy
mitmproxy --mode transparent --showhost

# Test if app detects MITM
# App should REFUSE connection with invalid cert

# ARP spoofing detection
arpwatch -i eth0

PASS CRITERIA:
[ ] App rejects self-signed certs
[ ] App rejects expired certs
[ ] App rejects wrong hostname certs
[ ] Certificate pinning prevents MITM
```

### 2. API Security Testing

#### 2.1 Authentication Testing
```bash
# Test API without credentials
curl -X GET https://paper-api.alpaca.markets/v2/account

# Test with invalid credentials
curl -X GET https://paper-api.alpaca.markets/v2/account \
  -H "APCA-API-KEY-ID: invalid" \
  -H "APCA-API-SECRET-KEY: invalid"

# Test with expired/revoked credentials
# (Use previously valid but revoked keys)

PASS CRITERIA:
[ ] 401 on missing credentials
[ ] 401 on invalid credentials
[ ] 401 on expired credentials
[ ] No credential info in error messages
```

#### 2.2 Input Fuzzing
```bash
# Fuzz order parameters
wfuzz -z file,/usr/share/wordlists/wfuzz/Injections/SQL.txt \
  -H "Content-Type: application/json" \
  -d '{"symbol":"FUZZ","qty":1,"side":"buy","type":"market"}' \
  http://localhost:1420/api/order

# Test boundary values
for qty in -1 0 0.0001 999999999 NaN Infinity; do
  echo "Testing qty=$qty"
  # Submit order with qty
done

PASS CRITERIA:
[ ] SQL injection blocked
[ ] XSS payloads sanitized
[ ] Negative quantities rejected
[ ] Overflow values rejected
[ ] NaN/Infinity handled
```

#### 2.3 Rate Limiting Test
```bash
# Rapid fire requests
for i in {1..100}; do
  curl -s -o /dev/null -w "%{http_code}\n" \
    http://localhost:1420/api/order &
done

# Check for 429 responses after threshold

PASS CRITERIA:
[ ] Rate limit enforced (429 after threshold)
[ ] Limit persists across restart (if applicable)
[ ] Different limits for different endpoints
```

### 3. Application Security Testing

#### 3.1 Memory Analysis
```bash
# Dump process memory
gcore $(pgrep financial-pipeline)

# Search for secrets in memory dump
strings core.* | grep -iE "(api.key|secret|password|private)"

# Monitor memory during operation
volatility -f memory.dump --profile=Linux imageinfo

PASS CRITERIA:
[ ] No plaintext API keys in memory
[ ] No plaintext passwords in memory
[ ] Secrets zeroed after use
```

#### 3.2 Binary Analysis
```bash
# Check for hardcoded secrets
strings target/release/financial-pipeline-gui | grep -iE "(key|secret|pass)"

# Check security features
checksec --file=target/release/financial-pipeline-gui

# Expected output:
# RELRO: Full RELRO
# Stack Canary: Canary found
# NX: NX enabled
# PIE: PIE enabled

PASS CRITERIA:
[ ] No hardcoded credentials
[ ] Full RELRO enabled
[ ] Stack canaries present
[ ] NX (non-executable stack) enabled
[ ] PIE enabled
[ ] No debug symbols in release
```

#### 3.3 Dependency Audit
```bash
# Rust dependency audit
cargo audit

# Check for known vulnerabilities
cargo deny check

# NPM audit for frontend
cd tauri-app && npm audit

PASS CRITERIA:
[ ] No critical vulnerabilities
[ ] No high vulnerabilities
[ ] All dependencies from trusted sources
[ ] Lock files committed
```

### 4. Local Storage Security

#### 4.1 SQLite Database Analysis
```bash
# Check for sensitive data
sqlite3 financial_data.db "SELECT * FROM sqlite_master;"
sqlite3 financial_data.db ".dump" | grep -iE "(key|secret|pass|token)"

# Check file permissions
ls -la financial_data.db

PASS CRITERIA:
[ ] No credentials in database
[ ] Database file permissions restricted (600)
[ ] No sensitive data in plaintext
```

#### 4.2 Config File Analysis
```bash
# Search for config files
find . -name "*.json" -o -name "*.toml" -o -name "*.yaml" | \
  xargs grep -l -iE "(key|secret|pass)"

# Check .env files
cat .env* 2>/dev/null

PASS CRITERIA:
[ ] No credentials in config files
[ ] .env files in .gitignore
[ ] Example configs don't contain real values
```

### 5. WebView/Frontend Security

#### 5.1 XSS Testing
```bash
# Test XSS in input fields
# Use Burp Suite or manual testing

PAYLOADS:
<script>alert('XSS')</script>
<img src=x onerror=alert('XSS')>
javascript:alert('XSS')
"><script>alert('XSS')</script>

PASS CRITERIA:
[ ] All user input escaped
[ ] CSP headers prevent inline scripts
[ ] No DOM-based XSS
```

#### 5.2 CORS/CSP Audit
```bash
# Check security headers
curl -I http://localhost:1420 | grep -iE "(content-security|x-frame|x-content)"

PASS CRITERIA:
[ ] Content-Security-Policy present
[ ] X-Frame-Options: DENY
[ ] X-Content-Type-Options: nosniff
[ ] No wildcard CORS
```

### 6. Crypto Wallet Security (If Applicable)

#### 6.1 Private Key Handling
```bash
# Check for exposed private keys
grep -r "0x[a-fA-F0-9]{64}" .
grep -r "private" . | grep -v ".git"

# Monitor for key exposure during signing
strace -f -e trace=write ./app 2>&1 | grep -i private

PASS CRITERIA:
[ ] No private keys in code
[ ] No private keys in logs
[ ] Keys loaded from secure storage only
[ ] Keys zeroed after transaction signing
```

#### 6.2 Transaction Simulation
```bash
# For DEX transactions, verify:
# - Simulation before execution
# - Slippage protection
# - Gas estimation accuracy

PASS CRITERIA:
[ ] Transaction simulated before broadcast
[ ] Slippage limits enforced
[ ] Gas estimate within 20% of actual
[ ] Failed simulation prevents execution
```

---

## API Security

### Alpaca API Security Implementation

```rust
// Secure API client implementation
use secrecy::{ExposeSecret, SecretString};
use reqwest::Certificate;

pub struct SecureAlpacaClient {
    api_key: SecretString,
    api_secret: SecretString,
    client: reqwest::Client,
}

impl SecureAlpacaClient {
    pub fn new() -> Result<Self, Error> {
        // Load from OS keyring
        let keyring = keyring::Entry::new("financial-pipeline", "alpaca-key")?;
        let api_key = SecretString::new(keyring.get_password()?);

        let keyring = keyring::Entry::new("financial-pipeline", "alpaca-secret")?;
        let api_secret = SecretString::new(keyring.get_password()?);

        // Build client with certificate pinning
        let cert = Certificate::from_pem(ALPACA_CERT_PEM)?;
        let client = reqwest::Client::builder()
            .add_root_certificate(cert)
            .min_tls_version(reqwest::tls::Version::TLS_1_3)
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self { api_key, api_secret, client })
    }

    pub async fn place_order(&self, order: &Order) -> Result<OrderResponse, Error> {
        // Validate order
        self.validate_order(order)?;

        // Create signed request
        let timestamp = Utc::now().timestamp();
        let signature = self.sign_request(order, timestamp);

        // Execute with retry logic
        let response = self.client
            .post("https://api.alpaca.markets/v2/orders")
            .header("APCA-API-KEY-ID", self.api_key.expose_secret())
            .header("APCA-API-SECRET-KEY", self.api_secret.expose_secret())
            .header("X-Timestamp", timestamp.to_string())
            .header("X-Signature", signature)
            .json(order)
            .send()
            .await?;

        // Audit log (no secrets)
        audit_log!("ORDER", order.symbol, order.qty, response.status());

        response.json().await
    }
}

impl Drop for SecureAlpacaClient {
    fn drop(&mut self) {
        // Secrets automatically zeroed by secrecy crate
    }
}
```

### Request Signing

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

fn sign_request(&self, order: &Order, timestamp: i64) -> String {
    let message = format!(
        "{}{}{}{}",
        timestamp,
        order.symbol,
        order.qty,
        order.side
    );

    let mut mac = Hmac::<Sha256>::new_from_slice(
        self.api_secret.expose_secret().as_bytes()
    ).expect("HMAC init failed");

    mac.update(message.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}
```

---

## Cryptographic Requirements

### Key Derivation
```
Algorithm: Argon2id
Memory: 64 MB
Iterations: 3
Parallelism: 4
Salt: 32 bytes random
Output: 32 bytes
```

### Encryption at Rest
```
Algorithm: AES-256-GCM
Key: Derived from master password via Argon2id
Nonce: 12 bytes random, never reused
Associated Data: Record metadata (non-secret)
```

### Secure Random Generation
```rust
use rand::rngs::OsRng;
use rand::RngCore;

fn generate_nonce() -> [u8; 12] {
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}
```

---

## Network Security

### Firewall Rules (Application Level)
```
ALLOW: HTTPS to api.alpaca.markets (port 443)
ALLOW: HTTPS to paper-api.alpaca.markets (port 443)
ALLOW: HTTPS to data.alpaca.markets (port 443)
ALLOW: WSS to stream.data.alpaca.markets (port 443)
DENY: All other outbound

For Crypto DEX:
ALLOW: HTTPS to mainnet RPC endpoints (configurable)
ALLOW: WSS to websocket endpoints (configurable)
```

### DNS Security
```
MUST: Use DNS-over-HTTPS or DNS-over-TLS
SHOULD: Pin known IP addresses as fallback
MUST: Verify hostname matches certificate
```

---

## Incident Response

### Severity Levels

| Level | Description | Response Time | Example |
|-------|-------------|---------------|---------|
| P0 | Active exploitation | Immediate | Unauthorized trades executing |
| P1 | Credential compromise | < 1 hour | API key leaked |
| P2 | Vulnerability found | < 24 hours | SQL injection discovered |
| P3 | Security improvement | < 1 week | Missing rate limit |

### Response Procedures

#### P0 - Active Exploitation
```
1. IMMEDIATELY: Revoke all API keys
2. IMMEDIATELY: Stop application
3. WITHIN 5 MIN: Contact broker to freeze account
4. WITHIN 1 HOUR: Forensic capture of logs/memory
5. WITHIN 24 HOURS: Root cause analysis
6. WITHIN 48 HOURS: Incident report
```

#### P1 - Credential Compromise
```
1. IMMEDIATELY: Rotate compromised credentials
2. WITHIN 1 HOUR: Audit recent activity for abuse
3. WITHIN 4 HOURS: Deploy fix to prevent recurrence
4. WITHIN 24 HOURS: Notify affected parties
```

### Kill Switch Implementation
```rust
// Emergency shutdown
pub fn emergency_stop() {
    // Cancel all pending orders
    cancel_all_orders().await;

    // Close all positions (optional, configurable)
    if config.emergency_liquidate {
        liquidate_all_positions().await;
    }

    // Revoke API session
    revoke_api_session().await;

    // Lock application
    app_state.lock();

    // Alert
    send_emergency_alert("KILL SWITCH ACTIVATED");
}
```

---

## Appendix A: Security Tools Reference

| Tool | Purpose | Command |
|------|---------|---------|
| testssl.sh | TLS configuration | `testssl.sh --severity HIGH <url>` |
| sqlmap | SQL injection | `sqlmap -u <url> --batch` |
| nikto | Web scanner | `nikto -h <url>` |
| nuclei | Vulnerability scanner | `nuclei -u <url> -t cves/` |
| cargo-audit | Rust deps | `cargo audit` |
| npm audit | JS deps | `npm audit` |
| checksec | Binary hardening | `checksec --file=<binary>` |
| gdb/gcore | Memory analysis | `gcore <pid>` |
| Burp Suite | Proxy/intercept | GUI |
| mitmproxy | MITM testing | `mitmproxy` |

---

## Appendix B: Compliance Notes

### Financial Regulations
- SEC Rule 15c3-5 (Market Access Rule) - Risk controls required
- FINRA Rule 3110 - Supervision requirements
- MiFID II (EU) - Best execution requirements
- ASIC (AU) - Market integrity rules

### Data Protection
- GDPR (EU) - Personal data handling
- CCPA (CA) - Consumer privacy
- Privacy Act 1988 (AU) - Information privacy

---

**END OF SPECIFICATION**

*This document should be reviewed and updated quarterly or after any security incident.*
