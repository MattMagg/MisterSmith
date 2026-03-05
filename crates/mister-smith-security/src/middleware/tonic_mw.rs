//! Tonic gRPC authentication interceptor.
//!
//! Validates JWT tokens from gRPC `authorization` metadata and injects
//! agent claims into request extensions.

use std::sync::Arc;

use tonic::{Request, Status};

use crate::middleware::SecurityLayer;

/// Create a Tonic interceptor closure that validates JWT tokens.
///
/// When security is disabled, all requests pass through unchanged.
/// When enabled, extracts the token from `authorization` metadata,
/// validates it, and inserts `AgentClaims` into request extensions.
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
                    security.audit.record_auth(
                        "unknown",
                        AuditOutcome::Failure,
                        [("reason".to_string(), "missing_authorization_metadata".to_string())]
                            .into_iter()
                            .collect(),
                    );
                }
                return Err(Status::unauthenticated("missing authorization metadata"));
            }
        };

        match security.jwt.validate_token(&token) {
            Ok(claims) => {
                #[cfg(feature = "audit")]
                {
                    use crate::audit::events::AuditOutcome;
                    security.audit.record_auth(
                        &claims.sub,
                        AuditOutcome::Success,
                        std::collections::HashMap::new(),
                    );
                }
                request.extensions_mut().insert(claims);
                Ok(request)
            }
            Err(e) => {
                #[cfg(feature = "audit")]
                {
                    use crate::audit::events::AuditOutcome;
                    security.audit.record_auth(
                        "unknown",
                        AuditOutcome::Failure,
                        [("reason".to_string(), e.to_string())]
                            .into_iter()
                            .collect(),
                    );
                }
                Err(Status::unauthenticated(e.to_string()))
            }
        }
    }
}
