use std::collections::HashMap;
use std::sync::Mutex;

use tokio_util::sync::CancellationToken;

#[derive(Default)]
pub struct SessionRegistry {
    tokens: Mutex<HashMap<String, CancellationToken>>,
}

impl SessionRegistry {
    pub fn register(&self, session_id: String) -> CancellationToken {
        let token = CancellationToken::new();
        self.tokens
            .lock()
            .unwrap()
            .insert(session_id, token.clone());
        token
    }

    pub fn cancel(&self, session_id: &str) {
        if let Some(token) = self.tokens.lock().unwrap().get(session_id) {
            token.cancel();
        }
    }

    pub fn remove(&self, session_id: &str) {
        self.tokens.lock().unwrap().remove(session_id);
    }
}
