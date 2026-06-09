use log::info;

pub fn authenticate(user: &str, password: &str) -> bool {
    info!("authentification for {}, {}", user, password);
    // TODO: authenticate client credentials against database
    // 1. Fetch user record from database
    // 2. Compare incoming 'pass' with the database hash (e.g., using argon2/bcrypt)
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_credentials() {
        let result = authenticate("admin", "secret123");
        assert!(result);
    }

    #[test]
    fn test_wrong_password() {
        let result = authenticate("admin", "wrong_password");
        assert!(!result); // Expecting false
    }
}
