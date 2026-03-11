## 2024-05-24 - [Overly Permissive CORS Headers]
**Vulnerability:** The HTTP server configures CORS to allow `Any` method and `Any` header.
**Learning:** `tower_http::cors::Any` permits any method and any header when CORS is configured. Even if `allow_origins` restricts origins, allowing any method or header may introduce security risks if the allowed origins include untrusted environments.
**Prevention:** Explicitly list the allowed HTTP methods and headers for CORS configurations instead of using `tower_http::cors::Any`.
