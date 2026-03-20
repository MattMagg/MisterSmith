## 2024-03-04 - [Missing Security Headers]
**Vulnerability:** [The application's HTTP transport layer lacked basic security headers like `X-Content-Type-Options: nosniff`]
**Learning:** [The application used a custom Axum router setup but didn't explicitly add security headers beyond CORS. This could allow MIME-sniffing attacks.]
**Prevention:** [Always add a middleware that explicitly sets security headers on all responses, particularly `X-Content-Type-Options: nosniff`]
