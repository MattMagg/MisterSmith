## 2024-05-25 - Hardcoded JWT secret in default configuration
**Vulnerability:** A hardcoded default JWT secret (`insecure-default-secret-change-me`) was found in `JwtConfig::default()`.
**Learning:** Hardcoding default secrets in structs often intended for quick prototyping can lead to these defaults leaking into production environments, allowing attackers to forge JWTs.
**Prevention:** Ensure cryptographic configurations always default to dynamically generating strong, securely random keys upon initialization (e.g. using `ring::rand`) if explicitly not provided.
