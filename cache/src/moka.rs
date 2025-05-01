use ::moka::future::Cache as MokaCache;
use std::hash::Hash;
use std::sync::Arc;

#[allow(clippy::module_inception)]
pub mod moka {
    pub use ::moka::future::*;
}

pub struct Cache<K: Hash + Eq, V> {
    inner: MokaCache<K, V>,
}

impl<K: Hash + Eq, V> Clone for Cache<K, V> {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl<K: Hash + Eq + Send + Sync + 'static, V: Send + Sync + 'static + Clone> Cache<K, V> {
    pub fn new(inner: MokaCache<K, V>) -> Self {
        Self { inner }
    }

    #[inline]
    pub async fn get(&self, key: &K) -> Option<V> {
        self.inner.get(key).await
    }

    #[inline]
    pub async fn get_or_insert_with<F: AsyncFnOnce() -> V>(&self, key: K, default: F) -> V {
        self.inner.entry(key).or_insert_with(async { default().await }).await.into_value()
    }

    #[inline]
    pub async fn get_or_try_insert_with<F: AsyncFnOnce() -> Result<V, E>, E: Send + Sync + 'static>(
        &self,
        key: K,
        default: F,
    ) -> Result<V, Arc<E>> {
        self.inner.entry(key).or_try_insert_with(async { default().await }).await.map(|v| v.into_value())
    }

    #[inline]
    pub async fn insert(&self, key: K, value: V) {
        self.inner.insert(key, value).await;
    }

    #[inline]
    pub async fn remove(&self, key: &K) -> Option<V> {
        self.inner.remove(key).await
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Duration;
    use thiserror::Error;

    fn new_cache<K: Hash + Eq + Send + Sync + 'static, V: Send + Sync + 'static + Clone>(ttl: Duration) -> Cache<K, V> {
        Cache::new(MokaCache::builder().time_to_live(ttl).build())
    }

    #[tokio::test]
    async fn should_accept_not_cloneable_key_and_value() {
        #[derive(Debug, Hash, Eq, PartialEq)]
        struct NotCloneable;

        let cache = new_cache(Duration::from_secs(1000));
        cache.insert(NotCloneable, 0u64).await;

        let cloned_cache = cache.clone();
        assert!(cache.get(&NotCloneable).await.is_some());
        assert!(cloned_cache.get(&NotCloneable).await.is_some());
    }

    #[tokio::test]
    async fn should_return_entry_not_expired() {
        let cache = new_cache(Duration::from_secs(1000));
        cache.insert("hello", "world").await;

        let result = cache.get(&"hello").await;

        assert!(result.is_some());
        assert_eq!("world", result.unwrap());
    }

    #[tokio::test]
    async fn should_not_insert_if_exists_on_get() {
        let cache = new_cache(Duration::from_secs(1000));
        cache.insert("hello", "world").await;

        let result = cache.get_or_insert_with("hello", || async { "new world!" }).await;

        assert_eq!("world", result);
    }

    #[tokio::test]
    async fn should_insert_on_get_if_not_present() {
        let cache = new_cache(Duration::from_secs(1000));

        let result = cache.get_or_insert_with(&"hello", || async { "new world" }).await;

        assert_eq!("new world", result);
    }

    #[tokio::test]
    async fn should_remove_entry() {
        let cache = new_cache(Duration::from_secs(1000));
        cache.insert("hello", "world").await;
        cache.remove(&"hello").await;
        assert!(cache.get(&"hello").await.is_none());
    }

    #[tokio::test]
    async fn should_not_try_insert_if_exists_on_get() {
        let cache = new_cache(Duration::from_secs(1000));
        cache.insert("hello", "world").await;

        let result = cache.get_or_try_insert_with("hello", insert_new_world_ok).await.unwrap();

        assert_eq!("world", result);
    }

    async fn insert_new_world_ok() -> Result<&'static str, TestError> {
        Ok("new world")
    }

    async fn insert_new_world_err() -> Result<&'static str, TestError> {
        Err(TestError::Error { message: "cannot insert" })
    }

    #[tokio::test]
    async fn should_try_insert_on_get_if_not_present() {
        let cache = new_cache(Duration::from_secs(1000));

        let result = cache.get_or_try_insert_with(&"hello", insert_new_world_ok).await.unwrap();

        assert_eq!("new world", result);
    }

    #[tokio::test]
    async fn should_try_insert_and_return_error() {
        let cache = new_cache(Duration::from_secs(1000));

        let result = cache.get_or_try_insert_with(&"hello", insert_new_world_err).await;

        match result {
            Ok(_) => panic!(),
            Err(e) => assert_eq!(Arc::new(TestError::Error { message: "cannot insert" }), e),
        }
    }

    #[derive(Error, Debug, PartialEq, Eq)]
    pub enum TestError {
        // JWT
        #[error("Error: [{message}]")]
        Error { message: &'static str },
    }

    #[tokio::test]
    async fn clone_should_link_to_same_map() {
        let cache = new_cache(Duration::from_secs(1000));
        let cloned_cache = cache.clone();

        cloned_cache.insert("hello", "world").await;
        assert!(cache.get(&"hello").await.is_some());

        cache.remove(&"hello").await;
        assert!(cloned_cache.get(&"hello").await.is_none());
    }
}
