# MisterSmith Documentation Workflows

These workflows automatically improve the MisterSmith framework documentation using both Claude (Anthropic) and OpenAI models.

## Required Secrets

You need to configure the following secrets in your GitHub repository settings:

1. **ANTHROPIC_API_KEY** - Already configured ✓
2. **OPENAI_API_KEY** - Required for OpenAI o4-mini-2025-04-16 model

### Setting up OPENAI_API_KEY

1. Go to Settings → Secrets and variables → Actions
2. Click "New repository secret"
3. Name: `OPENAI_API_KEY`
4. Value: Your OpenAI API key

## Workflow Model Distribution

### docs-async-runtime.yml
- **Tokio Runtime**: Claude Sonnet 4
- **Async Patterns**: OpenAI o4-mini-2025-04-16
- **Supervision Trees**: Claude Sonnet 4
- **Agent Lifecycle**: OpenAI o4-mini-2025-04-16

### docs-transport-messaging.yml
- **NATS Transport**: OpenAI o4-mini-2025-04-16
- **gRPC/Tonic**: Claude Sonnet 4
- **HTTP/Axum**: OpenAI o4-mini-2025-04-16
- **Message Schemas**: Claude Sonnet 4

### docs-data-persistence.yml
- **SQLx Patterns**: Claude Sonnet 4
- **Redis Caching**: OpenAI o4-mini-2025-04-16
- **Persistence Operations**: OpenAI o4-mini-2025-04-16
- **JetStream KV**: Claude Sonnet 4

### docs-security-crypto.yml
- **Authentication**: OpenAI o4-mini-2025-04-16
- **Authorization**: Claude Sonnet 4
- **Encryption**: OpenAI o4-mini-2025-04-16
- **Security Patterns**: Claude Sonnet 4

## Important Notes

- All workflows are restricted to only work with files in the `spec` directory
- Both models use Context7 to retrieve real code examples before converting to pseudocode
- OpenAI integration uses direct API calls since there's no GitHub Action for OpenAI yet
- Temperature is set to 0.3 for consistent, focused improvements