use std::sync::Arc;

use dashmap::DashMap;

#[derive(Clone)]
pub struct DefaultContext<T> {
    data: Arc<DashMap<String, T>>,
    field: Vec<(String, String)>,
}

impl<T> DefaultContext<T> {
    pub fn new() -> DefaultContext<T> {
        DefaultContext {
            data: Arc::new(DashMap::new()),
            field: Vec::new(),
        }
    }
}


impl<T> DefaultContext<T>
where
    T: Clone + Send + Sync,
{
    pub fn insert(&mut self, key: String, value: T) {
        self.data.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<T> {
        self.data.get(key).map(|v| v.clone())
    }

    pub fn remove(&mut self, key: &str) -> Option<T> {
        self.data.remove(key).map(|(_, v)| v.clone())
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    pub fn iter(&self) -> impl Iterator<Item = (String, T)> {
        self.data.iter().map(|item| (item.key().clone(), item.value().clone()))
    }

    pub fn keys(&self) -> impl Iterator<Item = String> {
        self.data.iter().map(|item| item.key().clone())
    }

    pub fn values(&self) -> impl Iterator<Item = T> {
        self.data.iter().map(|item| item.value().clone())
    }

    pub fn get_or_insert(&mut self, key: &str, value: T) -> T {
        self.data.entry(key.to_string()).or_insert(value.clone()).clone()
    }

    pub fn get_or_insert_with<F>(&mut self, key: &str, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.data.entry(key.to_string()).or_insert_with(f).clone()
    }
}

    