use chrono::prelude::Local;
use dashmap::{mapref::entry::Entry, DashMap};
use std::hash::Hash;
use std::sync::Arc;

pub struct Cache<K: Hash + Eq, V> {
    map: Arc<DashMap<K, (Arc<V>, i64)>>,
    ttl_ms: i64,
}

impl<K: Hash + Eq, V> Clone for Cache<K, V> {
    fn clone(&self) -> Self {
        Self { map: self.map.clone(), ttl_ms: self.ttl_ms }
    }
}

impl<K: Hash + Eq, V> Cache<K, V> {
    pub fn new(ttl_seconds: u32) -> Self {
        Self { map: Arc::new(DashMap::default()), ttl_ms: (ttl_seconds as i64) * 1000 }
    }

    pub fn get(&self, key: &K) -> Option<Arc<V>> {
        match self.map.get(key) {
            Some(value) => {
                if value.1 < current_epoch_mills() {
                    drop(value);
                    self.map.remove(key);
                    None
                } else {
                    Some(value.value().0.clone())
                }
            }
            None => None,
        }
    }

    pub async fn get_or_insert_with<F: FnOnce() -> Fut, Fut: std::future::Future<Output = V>>(
        &self,
        key: K,
        default: F,
    ) -> Arc<V> {
        match self.get(&key) {
            Some(value) => value,
            None => match self.map.entry(key) {
                Entry::Occupied(entry) => entry.into_ref().value().0.clone(),
                Entry::Vacant(entry) => {
                    let arc_value = Arc::new(default().await);
                    entry.insert(self.to_value(arc_value)).value().0.clone()
                }
            },
        }
    }

    pub async fn get_or_try_insert_with<F: FnOnce() -> Fut, Fut: std::future::Future<Output = Result<V, E>>, E>(
        &self,
        key: K,
        default: F,
    ) -> Result<Arc<V>, E> {
        match self.get(&key) {
            Some(value) => Ok(value),
            None => match self.map.entry(key) {
                Entry::Occupied(entry) => Ok(entry.into_ref().value().0.clone()),
                Entry::Vacant(entry) => {
                    let arc_value = Arc::new(default().await?);
                    let result = entry.insert(self.to_value(arc_value)).value().0.clone();
                    Ok(result)
                }
            },
        }
    }

    pub fn insert(&self, key: K, value: V) {
        self.map.insert(key, self.to_value(Arc::new(value)));
    }

    pub fn remove(&self, key: &K) {
        self.map.remove(key);
    }

    #[inline]
    fn to_value(&self, value: Arc<V>) -> (Arc<V>, i64) {
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
    async fn should_return_entry_not_expired() {
        let cache = Cache::new(1000);
        cache.insert("hello", "world");

        let result = cache.get(&"hello");

        assert!(result.is_some());
        assert_eq!(&"world", result.unwrap().as_ref());
    }

    #[tokio::test]
    async fn should_not_return_expired_entry() {
        let mut cache = Cache::new(1);
        cache.ttl_ms = 1;

        cache.insert("hello", "world");

        sleep(Duration::from_millis(2));

        let result = cache.get(&"hello");

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn should_not_insert_if_exists_on_get() {
        let cache = Cache::new(1000);
        cache.insert("hello", "world");

        let result = cache.get_or_insert_with("hello", || async { "new world!" }).await;

        assert_eq!(&"world", result.as_ref());
    }

    #[tokio::test]
    async fn should_insert_on_get_if_expired() {
        let mut cache = Cache::new(1);
        cache.ttl_ms = 1;

        cache.insert("hello", "world");

        sleep(Duration::from_millis(2));

        let result = cache.get_or_insert_with("hello", || async { "new world" }).await;

        assert_eq!(&"new world", result.as_ref());
    }

    #[tokio::test]
    async fn should_insert_on_get_if_not_present() {
        let cache = Cache::new(100);

        let result = cache.get_or_insert_with(&"hello", || async { "new world" }).await;

        assert_eq!(&"new world", result.as_ref());
    }

    #[tokio::test]
    async fn should_remove_entry() {
        let cache = Cache::new(1000);
        cache.insert("hello", "world");
        cache.remove(&"hello");
        assert!(cache.get(&"hello").is_none());
    }

    #[tokio::test]
    async fn should_not_try_insert_if_exists_on_get() {
        let cache = Cache::new(1000);
        cache.insert("hello", "world");

        let result = cache.get_or_try_insert_with("hello", insert_new_world_ok).await.unwrap();

        assert_eq!(&"world", result.as_ref());
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

        cache.insert("hello", "world");

        sleep(Duration::from_millis(2));

        let result = cache.get_or_try_insert_with("hello", insert_new_world_ok).await.unwrap();

        assert_eq!(&"new world", result.as_ref());
    }

    #[tokio::test]
    async fn should_try_insert_on_get_if_not_present() {
        let cache = Cache::new(100);

        let result = cache.get_or_try_insert_with(&"hello", insert_new_world_ok).await.unwrap();

        assert_eq!(&"new world", result.as_ref());
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

    #[test]
    fn should_not_overflow_when_ttl_is_max() {
        Cache::<String, String>::new(u32::MAX);
    }

    #[derive(Error, Debug, PartialEq, Eq)]
    pub enum TestError {
        // JWT
        #[error("Error: [{message}]")]
        Error { message: &'static str },
    }

    #[test]
    fn clone_should_link_to_same_map() {
        let cache = Cache::new(1000);
        let cloned_cache = cache.clone();

        cloned_cache.insert("hello", "world");
        assert!(cache.get(&"hello").is_some());

        cache.remove(&"hello");
        assert!(cloned_cache.get(&"hello").is_none());
    }
}
