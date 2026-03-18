//! Axum HTTP authentication and authorization middleware.
//!
//! Validates JWT Bearer tokens from the `Authorization` header, enforces
//! rate limiting, and injects [`AgentClaims`] into request extensions.

use std::sync::Arc;

use axum::extract::State;
use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::delegation::external_delegation_envelope;
use crate::jwt::AgentClaims;
use crate::middleware::SecurityLayer;
#[cfg(feature = "rbac")]
use crate::rbac::AuthorizationRequest;
use mister_smith_core::{
    CapabilityActionKind, DelegatedAction, DelegatedActionPolicy, DelegationScope,
    ExternalDelegationEnvelope, SecurityError,
};

/// Axum middleware that validates JWT Bearer tokens.
///
/// When security is disabled (master switch off), all requests pass through.
/// When enabled:
/// 1. Checks rate limiter — returns 429 if exceeded.
/// 2. Extracts Bearer token from `Authorization` header — returns 401 if missing.
/// 3. Validates the token via `JwtManager` — returns 401 if invalid/expired/revoked.
/// 4. Evaluates RBAC policy (when enabled) — returns 403 if unauthorized.
/// 5. Inserts `AgentClaims` into request extensions for downstream handlers.
pub async fn auth_middleware(
    State(security): State<Arc<SecurityLayer>>,
    mut request: Request<axum::body::Body>,
    next: Next,
) -> Response {
    if !security.is_enabled() {
        return next.run(request).await;
    }

    // Rate limiting — use peer address to prevent X-Forwarded-For spoofing
    let source = request
        .extensions()
        .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
        .map(|axum::extract::ConnectInfo(addr)| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    if let Err(retry_after) = security.rate_limiter.check(&source) {
        #[cfg(feature = "audit")]
        {
            use crate::audit::events::AuditOutcome;
            if let Some(audit) = security.audit.as_ref() {
                audit.record_auth(
                    &source,
                    AuditOutcome::Blocked,
                    [("reason".to_string(), "rate_limited".to_string())]
                        .into_iter()
                        .collect(),
                );
            }
        }
        return rate_limited_response(retry_after.as_secs());
    }

    let Some(jwt) = security.jwt.as_ref() else {
        return next.run(request).await;
    };

    // Extract Bearer token
    let token = match extract_bearer_token(&request) {
        Some(t) => t,
        None => {
            #[cfg(feature = "audit")]
            {
                use crate::audit::events::AuditOutcome;
                if let Some(audit) = security.audit.as_ref() {
                    audit.record_auth(
                        &source,
                        AuditOutcome::Failure,
                        [("reason".to_string(), "missing_auth_header".to_string())]
                            .into_iter()
                            .collect(),
                    );
                }
            }
            return unauthorized_response("missing authorization header");
        }
    };

    // Validate token
    match jwt.validate_token(&token) {
        Ok(claims) => {
            let delegation_envelope = match security.delegation_service.as_ref() {
                Some(delegation_service) => {
                    let boundary_action = build_http_delegated_action(&request);
                    let expected_descriptor = boundary_action.descriptor_id.clone();
                    match delegation_service.validate_claims_for_action(&claims, &boundary_action) {
                        Ok(validated) => match validated {
                            Some(validated)
                                if validated.capability.descriptor_id.as_deref()
                                    == Some(expected_descriptor.as_str()) =>
                            {
                                Some(external_delegation_envelope(
                                    &validated,
                                    Some(&boundary_action),
                                ))
                            }
                            Some(validated) => {
                                let reason = match validated.capability.descriptor_id.as_deref() {
                                    Some(descriptor_id) => format!(
                                        "delegation descriptor '{descriptor_id}' does not authorize HTTP boundary '{expected_descriptor}'"
                                    ),
                                    None => format!(
                                        "delegation descriptor missing for HTTP boundary '{expected_descriptor}'"
                                    ),
                                };
                                #[cfg(feature = "audit")]
                                {
                                    use crate::audit::events::AuditOutcome;
                                    if let Some(audit) = security.audit.as_ref() {
                                        audit.record_auth(
                                            &claims.sub,
                                            AuditOutcome::Failure,
                                            [("reason".to_string(), reason.clone())]
                                                .into_iter()
                                                .collect(),
                                        );
                                    }
                                }
                                return unauthorized_response("invalid delegation envelope");
                            }
                            None => None,
                        },
                        Err(error) => {
                            #[cfg(feature = "audit")]
                            {
                                use crate::audit::events::AuditOutcome;
                                if let Some(audit) = security.audit.as_ref() {
                                    audit.record_auth(
                                        &claims.sub,
                                        AuditOutcome::Failure,
                                        [("reason".to_string(), error.to_string())]
                                            .into_iter()
                                            .collect(),
                                    );
                                }
                            }
                            return unauthorized_response("invalid delegation envelope");
                        }
                    }
                }
                None => None,
            };

            #[cfg(feature = "audit")]
            {
                use crate::audit::events::AuditOutcome;
                if let Some(audit) = security.audit.as_ref() {
                    audit.record_auth(
                        &claims.sub,
                        AuditOutcome::Success,
                        std::collections::HashMap::new(),
                    );
                }
            }
            #[cfg(feature = "rbac")]
            {
                if let Some(policy) = security.policy.as_ref() {
                    let authz_request = build_http_authorization_request(&request, &claims);
                    let decision = policy.evaluate(&authz_request);

                    #[cfg(feature = "audit")]
                    {
                        use crate::audit::events::AuditOutcome;
                        if let Some(audit) = security.audit.as_ref() {
                            audit.record_authz(
                                &claims.sub,
                                &authz_request.action,
                                &authz_request.resource,
                                if decision.allowed {
                                    AuditOutcome::Success
                                } else {
                                    AuditOutcome::Failure
                                },
                            );
                        }
                    }

                    if !decision.allowed {
                        return forbidden_response("forbidden");
                    }
                }
            }

            request.extensions_mut().insert(claims);
            if let Some(envelope) = delegation_envelope {
                request.extensions_mut().insert(envelope);
            }
            next.run(request).await
        }
        Err(e) => {
            #[cfg(feature = "audit")]
            {
                use crate::audit::events::AuditOutcome;
                if let Some(audit) = security.audit.as_ref() {
                    audit.record_auth(
                        &source,
                        AuditOutcome::Failure,
                        [("reason".to_string(), e.to_string())]
                            .into_iter()
                            .collect(),
                    );
                }
            }
            unauthorized_response(map_auth_error_message(&e))
        }
    }
}

fn matched_route<B>(request: &Request<B>) -> String {
    request
        .extensions()
        .get::<axum::extract::MatchedPath>()
        .map(axum::extract::MatchedPath::as_str)
        .unwrap_or_else(|| request.uri().path())
        .to_string()
}

fn build_http_delegated_action<B>(request: &Request<B>) -> DelegatedAction {
    let route = matched_route(request);
    let method = request.method().as_str().to_ascii_lowercase();
    let descriptor_id = format!("http:{method}:{route}");
    let action_id = format!("{descriptor_id}#execute");

    DelegatedAction {
        descriptor_id,
        action_id: action_id.clone(),
        title: format!("execute {method} {route}"),
        description: format!("execute access for {method} {route}"),
        kind: CapabilityActionKind::Execute,
        policy: DelegatedActionPolicy {
            action: method,
            resource: "http".to_string(),
            scope: route.clone(),
            resource_id: Some(route),
        },
        required_scope: Some(DelegationScope::InvokeTool),
        revocation_key: action_id,
    }
}

#[cfg(feature = "rbac")]
fn build_http_authorization_request<B>(
    request: &Request<B>,
    claims: &AgentClaims,
) -> AuthorizationRequest {
    let route = matched_route(request);
    let action = request.method().as_str().to_ascii_lowercase();

    AuthorizationRequest {
        principal: claims.clone(),
        action,
        resource: route.clone(),
        resource_id: Some(route.clone()),
        context: [
            ("scope".to_string(), route),
            (
                "http_method".to_string(),
                request.method().as_str().to_string(),
            ),
            ("transport".to_string(), "http".to_string()),
        ]
        .into_iter()
        .collect(),
    }
}

fn map_auth_error_message(error: &SecurityError) -> &'static str {
    match error {
        SecurityError::TokenExpired => "token expired",
        SecurityError::TokenRevoked => "token revoked",
        _ => "unauthorized",
    }
}

/// Axum extractor for authenticated agent identity.
///
/// # Example
///
/// ```ignore
/// async fn handler(AuthenticatedAgent(claims): AuthenticatedAgent) -> impl IntoResponse {
///     format!("Hello, agent {}", claims.agent_id)
/// }
/// ```
pub struct AuthenticatedAgent(pub AgentClaims);

impl<S> axum::extract::FromRequestParts<S> for AuthenticatedAgent
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<AgentClaims>()
            .cloned()
            .map(AuthenticatedAgent)
            .ok_or_else(|| unauthorized_response("not authenticated"))
    }
}

/// Axum extractor for a validated external delegation envelope, when present.
pub struct AuthenticatedDelegation(pub Option<ExternalDelegationEnvelope>);

impl<S> axum::extract::FromRequestParts<S> for AuthenticatedDelegation
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(
            parts
                .extensions
                .get::<ExternalDelegationEnvelope>()
                .cloned(),
        ))
    }
}

/// Extract a Bearer token from the Authorization header.
fn extract_bearer_token<B>(request: &Request<B>) -> Option<String> {
    request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

/// Build a 401 Unauthorized JSON response.
fn unauthorized_response(message: &str) -> Response {
    let body = serde_json::json!({ "error": message });
    (StatusCode::UNAUTHORIZED, axum::Json(body)).into_response()
}

/// Build a 403 Forbidden JSON response.
fn forbidden_response(message: &str) -> Response {
    let body = serde_json::json!({ "error": message });
    (StatusCode::FORBIDDEN, axum::Json(body)).into_response()
}

/// Build a 429 Too Many Requests JSON response.
fn rate_limited_response(retry_after_secs: u64) -> Response {
    let body = serde_json::json!({
        "error": "rate limit exceeded",
        "retry_after": retry_after_secs,
    });
    let mut response = (StatusCode::TOO_MANY_REQUESTS, axum::Json(body)).into_response();
    if let Ok(val) = axum::http::HeaderValue::from_str(&retry_after_secs.to_string()) {
        response.headers_mut().insert("retry-after", val);
    }
    response
}
