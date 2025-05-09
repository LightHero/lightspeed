use chrono::prelude::Local;
use std::hash::Hash;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{RwLock, RwLockWriteGuard};

type InnerMap<K, V> = Arc<RwLock<HashMap<K, (V, i64)>>>;

#[derive(Debug)]
pub struct Cache<K: Hash + Eq, V> {
    map: InnerMap<K, V>,
    ttl_ms: i64,
}

impl<K: Hash + Eq, V> Clone for Cache<K, V> {
    fn clone(&self) -> Self {
        Self { map: self.map.clone(), ttl_ms: self.ttl_ms }
    }
}

impl<K: Hash + Eq, V: Clone> Cache<K, V> {
    pub fn new(ttl_seconds: u32) -> Self {
        Self { map: Arc::new(RwLock::new(HashMap::default())), ttl_ms: (ttl_seconds as i64) * 1000 }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let read = self.map.read().await;
        match read.get(key) {
            Some(value) => {
                if value.1 < current_epoch_mills() {
                    drop(read);
                    self.remove(key).await;
                    None
                } else {
                    Some(value.0.clone())
                }
            }
            None => None,
        }
    }

    pub async fn get_or_insert_with<F: AsyncFnOnce() -> V>(&self, key: K, default: F) -> V {
        match self.get(&key).await {
            Some(value) => value,
            None => {
                let write = self.map.write().await;
                match write.get(&key) {
                    Some(val) => val.0.clone(),
                    None => {
                        let new_value = default().await;
                        self.insert_to_guard(write, key, new_value.clone());
                        new_value
                    }
                }
            }
        }
    }

    pub async fn get_or_try_insert_with<F: AsyncFnOnce() -> Result<V, E>, E>(
        &self,
        key: K,
        default: F,
    ) -> Result<V, E> {
        match self.get(&key).await {
            Some(value) => Ok(value),
            None => {
                let write = self.map.write().await;
                match write.get(&key) {
                    Some(val) => Ok(val.0.clone()),
                    None => {
                        let new_value = default().await?;
                        self.insert_to_guard(write, key, new_value.clone());
                        Ok(new_value)
                    }
                }
            }
        }
    }

    pub async fn insert(&self, key: K, value: V) {
        let write = self.map.write().await;
        self.insert_to_guard(write, key, value);
    }

    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut write = self.map.write().await;
        write.remove(key).map(|v| v.0)
    }

    fn insert_to_guard(&self, mut write: RwLockWriteGuard<'_, HashMap<K, (V, i64)>>, key: K, value: V) {
        write.insert(key, self.to_value(value));
    }

    #[inline]
    fn to_value(&self, value: V) -> (V, i64) {
        (value, current_epoch_mills() + self.ttl_ms)
    }
}

fn current_epoch_mills() -> i64 {
    Local::now().timestamp_millis()
}

#[cfg(test)]
mod test {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;
    use thiserror::Error;

    #[tokio::test]
    async fn should_accept_not_cloneable_key_and_value() {
        #[derive(Debug, Hash, Eq, PartialEq)]
        struct NotCloneable;

        let cache = Cache::<NotCloneable, u64>::new(1000);
        cache.insert(NotCloneable, 3).await;

        let cloned_cache = cache.clone();
        assert!(cache.get(&NotCloneable).await.is_some());
        assert!(cloned_cache.get(&NotCloneable).await.is_some());
    }

    #[tokio::test]
    async fn should_return_entry_not_expired() {
        let cache = Cache::new(1000);
        cache.insert("hello", "world").await;

        let result = cache.get(&"hello").await;

        assert!(result.is_some());
        assert_eq!("world", result.unwrap());
    }

    #[tokio::test]
    async fn should_not_return_expired_entry() {
        let mut cache = Cache::new(1);
        cache.ttl_ms = 1;

        cache.insert("hello", "world").await;

        sleep(Duration::from_millis(2));

        let result = cache.get(&"hello").await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_not_insert_if_exists_on_get() {
        let cache = Cache::new(1000);
        cache.insert("hello", "world").await;

        let result = cache.get_or_insert_with("hello", async || "new world!").await;

        assert_eq!("world", result);
    }

    #[tokio::test]
    async fn should_insert_on_get_if_expired() {
        let mut cache = Cache::new(1);
        cache.ttl_ms = 1;

        cache.insert("hello", "world").await;

        sleep(Duration::from_millis(2));

        let result = cache.get_or_insert_with("hello", async || "new world").await;

        assert_eq!("new world", result);
    }

    #[tokio::test]
    async fn should_insert_on_get_if_not_present() {
        let cache = Cache::new(100);

        let result = cache.get_or_insert_with(&"hello", async || "new world").await;

        assert_eq!("new world", result);
    }

    #[tokio::test]
    async fn should_remove_entry() {
        let cache = Cache::new(1000);
        cache.insert("hello", "world").await;
        cache.remove(&"hello").await;
        assert!(cache.get(&"hello").await.is_none());
    }

    #[tokio::test]
    async fn should_not_try_insert_if_exists_on_get() {
        let cache = Cache::new(1000);
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
    async fn should_try_insert_on_get_if_expired() {
        let mut cache = Cache::new(1);
        cache.ttl_ms = 1;

        cache.insert("hello", "world").await;

        sleep(Duration::from_millis(2));

        let result = cache.get_or_try_insert_with("hello", insert_new_world_ok).await.unwrap();

        assert_eq!("new world", result);
    }

    #[tokio::test]
    async fn should_try_insert_on_get_if_not_present() {
        let cache = Cache::new(100);

        let result = cache.get_or_try_insert_with(&"hello", insert_new_world_ok).await.unwrap();

        assert_eq!("new world", result);
    }

    #[tokio::test]
    async fn should_try_insert_and_return_error() {
        let cache = Cache::new(100);

        let result = cache.get_or_try_insert_with(&"hello", insert_new_world_err).await;

        match result {
            Ok(_) => panic!(),
            Err(e) => assert_eq!(TestError::Error { message: "cannot insert" }, e),
        }
    }

    #[tokio::test]
    async fn should_not_overflow_when_ttl_is_max() {
        Cache::<String, String>::new(u32::MAX);
    }

    #[derive(Error, Debug, PartialEq, Eq)]
    pub enum TestError {
        // JWT
        #[error("Error: [{message}]")]
        Error { message: &'static str },
    }

    #[tokio::test]
    async fn clone_should_link_to_same_map() {
        let cache = Cache::new(1000);
        let cloned_cache = cache.clone();

        cloned_cache.insert("hello", "world").await;
        assert!(cache.get(&"hello").await.is_some());

        cache.remove(&"hello").await;
        assert!(cloned_cache.get(&"hello").await.is_none());
    }
}
