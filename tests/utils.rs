/// A simple RAII guard that resets an env var after the test exits
pub struct EnvVarGuard {
    key: String,
    original: Option<String>,
}

impl EnvVarGuard {
    pub fn new<K: Into<String>, V: Into<String>>(key: K, value: V) -> Self {
        let key = key.into();
        let original = std::env::var(&key).ok();
        std::env::set_var(&key, value.into());
        Self { key, original }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(ref val) = self.original {
            std::env::set_var(&self.key, val);
        } else {
            std::env::remove_var(&self.key);
        }
    }
}
