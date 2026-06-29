use log::info;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

static USERS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn authenticate(user: &str, password: &str) -> bool {
    let mut map = USERS.lock().unwrap();
    if let Some(stored) = map.get(user) {
        if stored == password {
            info!("Authenticated {user}");
            true
        } else {
            info!("Authentication failed for {user}");
            false
        }
    } else {
        info!("Auto-registering new user {user}");
        map.insert(user.to_string(), password.to_string());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_register_and_authenticate() {
        assert!(authenticate("alice", "secret123")); // auto-register
        assert!(authenticate("alice", "secret123")); // correct password
        assert!(!authenticate("alice", "wrong")); // wrong password
    }
}
