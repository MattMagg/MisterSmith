//! MCP Resource registry for exposing agent knowledge and config.
//!
//! Registers agent knowledge bases and configuration snapshots as
//! read-only MCP Resources. Excludes runtime state (mailbox depth,
//! supervision status) per spec requirements.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::errors::McpError;

/// A registered MCP resource (read-only).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McpResource {
    /// Unique resource URI (e.g., "agent://worker-1/config").
    pub uri: String,
    /// Human-readable name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// MIME type of the resource content.
    pub mime_type: String,
    /// Resource content (read-only snapshot).
    content: String,
}

impl McpResource {
    /// Create a new MCP resource.
    pub fn new(uri: &str, name: &str, mime_type: &str, content: String) -> Self {
        Self {
            uri: uri.to_string(),
            name: name.to_string(),
            description: None,
            mime_type: mime_type.to_string(),
            content,
        }
    }

    /// Create a JSON resource.
    pub fn json(uri: &str, name: &str, content: &serde_json::Value) -> Self {
        Self {
            uri: uri.to_string(),
            name: name.to_string(),
            description: None,
            mime_type: "application/json".to_string(),
            content: content.to_string(),
        }
    }

    /// Create a text resource.
    pub fn text(uri: &str, name: &str, content: &str) -> Self {
        Self::new(uri, name, "text/plain", content.to_string())
    }

    /// Set the description.
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    /// Get the resource content.
    pub fn content(&self) -> &str {
        &self.content
    }
}

/// Registry for MCP resources (agent knowledge bases and config snapshots).
pub struct ResourceRegistry {
    /// Resources keyed by URI.
    resources: Arc<RwLock<HashMap<String, McpResource>>>,
}

impl ResourceRegistry {
    /// Create a new empty resource registry.
    pub fn new() -> Self {
        Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a resource. Overwrites if URI already exists.
    pub async fn register(&self, resource: McpResource) {
        let mut resources = self.resources.write().await;
        resources.insert(resource.uri.clone(), resource);
    }

    /// Remove a resource by URI.
    pub async fn unregister(&self, uri: &str) -> bool {
        let mut resources = self.resources.write().await;
        resources.remove(uri).is_some()
    }

    /// Handle `resources/list` — return all registered resources.
    pub async fn list(&self) -> Vec<McpResource> {
        let resources = self.resources.read().await;
        resources.values().cloned().collect()
    }

    /// Handle `resources/read` — return resource content by URI.
    pub async fn read(&self, uri: &str) -> Result<McpResource, McpError> {
        let resources = self.resources.read().await;
        resources
            .get(uri)
            .cloned()
            .ok_or_else(|| McpError::ToolNotFound(format!("resource not found: {uri}")))
    }

    /// Get the count of registered resources.
    pub async fn count(&self) -> usize {
        let resources = self.resources.read().await;
        resources.len()
    }
}

impl Default for ResourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn register_and_list() {
        let registry = ResourceRegistry::new();

        registry
            .register(McpResource::text(
                "agent://test/readme",
                "README",
                "Hello world",
            ))
            .await;

        let resources = registry.list().await;
        assert_eq!(resources.len(), 1);
        assert_eq!(resources[0].uri, "agent://test/readme");
        assert_eq!(resources[0].content(), "Hello world");
    }

    #[tokio::test]
    async fn read_resource() {
        let registry = ResourceRegistry::new();

        let config = serde_json::json!({"debug": true, "workers": 4});
        registry
            .register(
                McpResource::json("agent://worker/config", "Worker Config", &config)
                    .with_description("Worker configuration snapshot"),
            )
            .await;

        let res = registry.read("agent://worker/config").await.unwrap();
        assert_eq!(res.name, "Worker Config");
        assert_eq!(res.mime_type, "application/json");
        assert!(res.description.is_some());
    }

    #[tokio::test]
    async fn read_missing_returns_error() {
        let registry = ResourceRegistry::new();
        let result = registry.read("agent://missing").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn unregister_resource() {
        let registry = ResourceRegistry::new();
        registry
            .register(McpResource::text("agent://test/data", "Data", "content"))
            .await;
        assert_eq!(registry.count().await, 1);

        let removed = registry.unregister("agent://test/data").await;
        assert!(removed);
        assert_eq!(registry.count().await, 0);

        let removed_again = registry.unregister("agent://test/data").await;
        assert!(!removed_again);
    }

    #[tokio::test]
    async fn overwrite_existing() {
        let registry = ResourceRegistry::new();

        registry
            .register(McpResource::text("agent://test/v", "V1", "version 1"))
            .await;
        registry
            .register(McpResource::text("agent://test/v", "V2", "version 2"))
            .await;

        assert_eq!(registry.count().await, 1);
        let res = registry.read("agent://test/v").await.unwrap();
        assert_eq!(res.name, "V2");
        assert_eq!(res.content(), "version 2");
    }
}
