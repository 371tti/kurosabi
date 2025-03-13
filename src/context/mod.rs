pub trait Context {
    fn get(&self, key: &str) -> Option<&str>;
    fn set(&self, key: &str, value: &str) -> ();
    fn remove(&self, key: &str) -> ();
    fn clear(&self) -> ();
    fn keys(&self) -> Vec<&str>;
    fn values(&self) -> Vec<&str>;
    fn iter(&self) -> Box<dyn Iterator<Item = (&str, &str)>>;
    fn len(&self) -> usize;
}
    