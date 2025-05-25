#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    async fn get(&self, key: &str) -> eyre::Result<Option<String>>;
    async fn set(&self, key: &str, value: String) -> eyre::Result<()>;
    async fn delete(&self, key: &str) -> eyre::Result<()>;
}