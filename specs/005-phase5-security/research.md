# Research: Phase 5 Security

**Feature Branch**: `005-phase5-security`
**Date**: 2026-03-04
**Status**: Complete

## Research Tasks

### R1: jsonwebtoken 10.x API (Decision: Use jsonwebtoken 10.3.0 with `aws_lc_rs` feature)

**Rationale**: jsonwebtoken 10 requires an explicit crypto backend selection — no default is provided. `aws_lc_rs` is the recommended backend for production use and aligns with the rustls CryptoProvider choice for consistency across the security stack.

**Key API findings**:

- **Encoding**: `jsonwebtoken::encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)` → `Result<String>`
- **Decoding**: `jsonwebtoken::decode::<T>(token, &decoding_key, &validation)` → `Result<TokenData<T>>`
- **TokenData<T>**: Contains `.header: Header` and `.claims: T`
- **Algorithm enum**: 12 variants — HS256/384/512, RS256/384/512, PS256/384/512, ES256/384, EdDSA
- **EncodingKey construction**:
  - `EncodingKey::from_secret(secret)` — HMAC
  - `EncodingKey::from_rsa_pem(pem)` — RSA from PEM
  - `EncodingKey::from_ec_pem(pem)` — ECDSA from PEM
  - `EncodingKey::from_ed_pem(pem)` — EdDSA from PEM
- **DecodingKey construction**: Mirrors EncodingKey with `from_secret`, `from_rsa_pem`, `from_ec_pem`, `from_ed_pem`, plus `from_rsa_components(n, e)` and `from_jwk(jwk)`
- **Validation** struct: Configure `leeway`, `validate_exp`, `validate_nbf`, `validate_aud`, `aud` (required audiences), `iss` (required issuers), `sub`, `algorithms` (allowed algorithms)
- **Breaking changes from v9**: Crypto backend is trait-based via `CryptoProvider`; must select `aws_lc_rs` or `rust_crypto` feature. No other API-breaking changes.

**Alternatives considered**:
- `jwt-simple 0.12.x`: Simpler API but less control over claim structure and validation. Staying with jsonwebtoken for ecosystem consistency and custom `AgentClaims` support.
- `rust_crypto` feature: Viable alternative backend, but `aws_lc_rs` is more performant and already required by the rustls stack.

### R2: rustls 0.23.x + tokio-rustls 0.26.x + rcgen 0.14.x API (Decision: Use aws_lc_rs CryptoProvider across the stack)

**Rationale**: rustls 0.23 requires explicit CryptoProvider selection. Using `aws_lc_rs` aligns with jsonwebtoken's backend choice, avoiding duplicate crypto libraries.

**Key API findings**:

**rustls 0.23.37**:
- **ServerConfig builder**: `ServerConfig::builder_with_provider(Arc::new(provider)).with_protocol_versions(&[&version::TLS13])?.with_client_cert_verifier(verifier).with_single_cert(certs, key)?`
- **ClientConfig builder**: `ClientConfig::builder_with_provider(Arc::new(provider)).with_protocol_versions(&[&version::TLS13])?.with_root_certificates(store).with_client_auth_cert(certs, key)?`
- **mTLS**: `WebPkiClientVerifier::builder_with_provider(Arc::new(roots), provider).build()?` replaces the old `AllowAnyAuthenticatedClient`
- **Certificate types**: `CertificateDer<'static>` and `PrivateKeyDer<'static>` from `rustls::pki_types` (old `Certificate`/`PrivateKey` wrappers are gone)
- **PEM loading**: `PemObject` trait provides `from_pem_file()`, `pem_file_iter()`, `from_pem_slice()`, `pem_slice_iter()` on `CertificateDer` and `PrivateKeyDer`
- **TLS versions**: `rustls::version::TLS12` and `rustls::version::TLS13` (type `&'static SupportedProtocolVersion`)
- **No client auth**: `.with_no_client_auth()` on both server and client builders

**tokio-rustls 0.26.4**:
- **TlsAcceptor**: `TlsAcceptor::from(Arc::new(server_config))`, then `acceptor.accept(tcp_stream).await?` → `TlsStream<TcpStream>`
- **TlsConnector**: `TlsConnector::from(Arc::new(client_config))`, then `connector.connect(domain, tcp_stream).await?`
- **ServerName**: `ServerName::try_from("hostname")?.to_owned()` for client connections
- **Important**: Must call `flush()` after writes — internal buffering does not auto-flush

**rcgen 0.14.7**:
- **KeyPair**: `KeyPair::generate()?` (default ECDSA P256), `KeyPair::generate_for(alg)?` for specific algorithms
- **CA certificate**: `CertificateParams::new(sans)?.self_signed(&key)?` with `is_ca = IsCa::Ca(BasicConstraints::Unconstrained)`
- **End-entity certificate**: `CertificateParams::new(sans)?.signed_by(&key, &issuer)?`
- **Issuer**: `Issuer::from_params(&ca_params, &ca_key)` or `Issuer::from_ca_cert_der(&der, &key)?`
- **Output**: `cert.pem()` → String, `cert.der()` → `&CertificateDer`, `key.serialize_pem()` → String
- **Simple one-liner**: `generate_simple_self_signed(sans)?` → `CertifiedKey { cert, key_pair }`

**Alternatives considered**:
- `ring` CryptoProvider: Simpler build (no cmake), but `aws_lc_rs` is recommended for production and avoids duplicating crypto backends since jsonwebtoken already uses it.
- `native-tls`: Not viable — doesn't support TLS 1.3-only enforcement or mTLS with the control needed.

### R3: Axum 0.8 / Tonic 0.14 Auth Middleware (Decision: Use existing patterns)

**Rationale**: Both frameworks have well-established middleware/interceptor patterns that align with the existing Phase 4 middleware architecture.

**Key API findings**:

**Axum 0.8 middleware**:
- **Layer approach**: `middleware::from_fn(auth_middleware)` or `from_fn_with_state(state, auth_middleware)` — already used in Phase 4 `security_middleware` placeholder
- **Extractor approach**: `FromRequestParts` trait (no `#[async_trait]` needed in Axum 0.8) for extracting authenticated identity from request extensions
- **Route exemption**: Split routes into authenticated and unauthenticated `Router`s, merge with `Router::merge()`. Only apply `route_layer(middleware::from_fn(...))` to the authenticated router.
- **Request extensions**: After validation, insert identity into `request.extensions_mut().insert(agent_claims)` for downstream handlers to extract via `Extension<AgentClaims>` extractor
- **Error responses**: Return `(StatusCode::UNAUTHORIZED, "msg").into_response()` for auth failures, `(StatusCode::FORBIDDEN, "msg").into_response()` for authz failures

**Tonic 0.14 interceptor**:
- **Interceptor trait**: `fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status>`
- **Token extraction**: `request.metadata().get("authorization")` → `Option<&MetadataValue<Ascii>>`
- **Per-service auth**: `ServiceBuilder::new().layer(InterceptorLayer::new(auth_interceptor)).service(my_service)` or `my_service_server.with_interceptor(auth_fn)`
- **Error responses**: Return `Err(Status::unauthenticated("msg"))` or `Err(Status::permission_denied("msg"))`
- **Request extensions**: `request.extensions_mut().insert(claims)` for downstream access

**NATS application-level enforcement**:
- No built-in NATS middleware — enforcement is at the application layer
- Before `transport.publish(subject, envelope)`, check RBAC permissions via `PolicyEngine::evaluate(principal, action, subject)`
- Wrap publish/subscribe in a `SecureTransport` layer that delegates to RBAC before forwarding to the underlying `Transport`

**Alternatives considered**:
- Tower middleware directly: More flexible but Axum's `from_fn` is simpler and already used in Phase 4.
- Custom gRPC service wrapper: Tonic's built-in `Interceptor` trait is sufficient and idiomatic.

## Dependency Matrix

```toml
# Phase 5 new dependencies
jsonwebtoken = { version = "10.3.0", features = ["aws_lc_rs"] }
rustls = "0.23.37"
tokio-rustls = "0.26.4"
rcgen = { version = "0.14.7", features = ["pem"] }
rustls-pki-types = "1"

# Already in workspace (used by Phase 5)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = "0.4"
uuid = { version = "1", features = ["v4"] }
tokio = { version = "1.49.0", features = ["full"] }
tracing = "0.1"
dashmap = "6"
```

## Integration Points (Existing Codebase)

| Component | Current State | Phase 5 Action |
|-----------|--------------|----------------|
| `SecurityConfig` in `mister-smith-config` | Placeholder: `enabled`, `tls_enabled`, `auth_required` | Expand with JWT, RBAC, TLS, audit subsystem configs |
| `security_middleware` in `mister-smith-http` | Pass-through stub with "Phase 5" comment | Replace with JWT validation + RBAC enforcement |
| `GrpcServer` in `mister-smith-grpc` | No auth interceptor | Add Tonic interceptor via `with_interceptor` |
| `SystemError` in `mister-smith-core` | 11 domain error types, no security variant | Add `SecurityError` variant with `#[from]` |
| `EventBus` in `mister-smith-events` | Working pub/sub for system events | Publish security audit events |
| `Transport` trait in `mister-smith-transport` | `publish`, `subscribe`, `request` | Wrap with `SecureTransport` for NATS RBAC |
