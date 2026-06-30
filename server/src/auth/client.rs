use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use log::info;
use rand::rngs::OsRng;
use std::collections::HashMap;
use std::sync::LazyLock;
use std::sync::Mutex;

static USERS: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn authenticate(user: &str, password: &str) -> bool {
    let mut map = USERS.lock().unwrap();
    // If user exists in map, verify password... otherwise register it.
    if let Some(stored) = map.get(user) {
        let parsed = PasswordHash::new(stored);
        if let Ok(parsed) = parsed {
            if Argon2::default()
                .verify_password(password.as_bytes(), &parsed)
                .is_ok()
            {
                info!("Authenticated {user}");
                return true;
            }
        }
    } else {
        info!("Auto-registering new user {user}");
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        if let Ok(password_hash) = argon2.hash_password(password.as_bytes(), &salt) {
            info!("Saving hash {} for {user}", password_hash);
            map.insert(user.to_string(), password_hash.to_string());
            return true;
        } else {
            info!("Password for {user} couldn't be hashed");
            return false;
        }
    }
    info!("Authentication failed for {user}");
    false
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
