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

pub enum AuthResult {
    /// New user was auto-registered
    Registered,
    /// Existing user, password matches
    Authenticated,
    /// Wrong password
    Denied,
}

pub fn authenticate(user: &str, password: &str) -> AuthResult {
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
                AuthResult::Authenticated
            } else {
                AuthResult::Denied
            }
        } else {
            AuthResult::Denied
        }
    } else {
        info!("Auto-registering new user {user}");
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        if let Ok(password_hash) = argon2.hash_password(password.as_bytes(), &salt) {
            info!("Saving hash {} for {user}", password_hash);
            map.insert(user.to_string(), password_hash.to_string());
            AuthResult::Registered
        } else {
            info!("Password for {user} couldn't be hashed");
            AuthResult::Denied
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auto_register_and_authenticate() {
        assert!(matches!(
            authenticate("alice", "secret123"),
            AuthResult::Registered
        ));
        assert!(matches!(
            authenticate("alice", "secret123"),
            AuthResult::Authenticated
        ));
        assert!(matches!(authenticate("alice", "wrong"), AuthResult::Denied));
    }
}
