//! Tonic gRPC authentication interceptor.
//!
//! Validates JWT tokens from gRPC `authorization` metadata and injects
//! agent claims into request extensions.

use std::sync::Arc;

use tonic::{Request, Status};

use crate::middleware::SecurityLayer;
#[cfg(feature = "rbac")]
use crate::rbac::AuthorizationRequest;
use mister_smith_core::SecurityError;

/// Create a Tonic interceptor closure that validates JWT tokens.
///
/// When security is disabled, all requests pass through unchanged.
/// When enabled, extracts the token from `authorization` metadata,
/// validates it, evaluates RBAC policy (when enabled), and inserts
/// `AgentClaims` into request extensions.
///
/// # Usage
///
/// ```ignore
/// let interceptor = grpc_auth_interceptor(security.clone());
/// let svc = MyServiceServer::with_interceptor(my_service, interceptor);
/// ```
pub fn grpc_auth_interceptor(
    security: Arc<SecurityLayer>,
) -> impl Fn(Request<()>) -> Result<Request<()>, Status> + Clone {
    move |mut request: Request<()>| {
        if !security.is_enabled() {
            return Ok(request);
        }

        let Some(jwt) = security.jwt.as_ref() else {
            return Ok(request);
        };

        // Extract token from metadata
        let token = request
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .map(|s| s.to_string());

        let token = match token {
            Some(t) => t,
            None => {
                #[cfg(feature = "audit")]
                {
                    use crate::audit::events::AuditOutcome;
                    if let Some(audit) = security.audit.as_ref() {
                        audit.record_auth(
                            "unknown",
                            AuditOutcome::Failure,
                            [(
                                "reason".to_string(),
                                "missing_authorization_metadata".to_string(),
                            )]
                            .into_iter()
                            .collect(),
                        );
                    }
                }
                return Err(Status::unauthenticated("missing authorization metadata"));
            }
        };

        match jwt.validate_token(&token) {
            Ok(claims) => {
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
                        let authz_request = build_grpc_authorization_request(&request, &claims);
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
                            return Err(Status::permission_denied("forbidden"));
                        }
                    }
                }

                request.extensions_mut().insert(claims);
                Ok(request)
            }
            Err(e) => {
                #[cfg(feature = "audit")]
                {
                    use crate::audit::events::AuditOutcome;
                    if let Some(audit) = security.audit.as_ref() {
                        audit.record_auth(
                            "unknown",
                            AuditOutcome::Failure,
                            [("reason".to_string(), e.to_string())]
                                .into_iter()
                                .collect(),
                        );
                    }
                }
                Err(Status::unauthenticated(map_auth_error_message(&e)))
            }
        }
    }
}

#[cfg(feature = "rbac")]
fn build_grpc_authorization_request(
    request: &Request<()>,
    claims: &crate::jwt::AgentClaims,
) -> AuthorizationRequest {
    let grpc_method = request.extensions().get::<tonic::GrpcMethod<'static>>();
    let service = grpc_method.map(|m| m.service()).unwrap_or("unknown");
    let method = grpc_method.map(|m| m.method()).unwrap_or("unknown");
    let path = format!("/{service}/{method}");

    AuthorizationRequest {
        principal: claims.clone(),
        action: "grpc_call".to_string(),
        resource: path.clone(),
        resource_id: Some(path.clone()),
        context: [
            ("scope".to_string(), path.clone()),
            ("grpc_method".to_string(), path),
            ("transport".to_string(), "grpc".to_string()),
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
