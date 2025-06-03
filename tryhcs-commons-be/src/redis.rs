use redis::AsyncCommands;

#[async_trait::async_trait]
pub trait Cache: Send + Sync {
    async fn get_key(&self, key: &str) -> eyre::Result<Option<String>>;
    async fn set_key(
        &self,
        key: &str,
        value: &str,
        duration_in_sec: Option<u64>,
    ) -> eyre::Result<()>;
}

#[derive(Clone)]
pub struct RedisCache(pub redis::Client);

#[async_trait::async_trait]
impl Cache for RedisCache {
    async fn get_key(&self, key: &str) -> eyre::Result<Option<String>> {
        let RedisCache(redis) = self;
        let mut con = redis.get_multiplexed_async_connection().await?;
        let value = con.get::<&str, Option<String>>(key).await?;
        Ok(value)
    }

    async fn set_key(
        &self,
        key: &str,
        value: &str,
        duration_in_sec: Option<u64>,
    ) -> eyre::Result<()> {
        let RedisCache(redis) = self;
        let mut con = redis.get_multiplexed_async_connection().await?;
        if let Some(timeout) = duration_in_sec {
            con.set_ex::<&str, &str, ()>(key, value, timeout as u64)
                .await?;
            return Ok(());
        }
        con.set::<&str, &str, ()>(key, value).await?;
        return Ok(());
    }
}
