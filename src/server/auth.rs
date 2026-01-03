//! Token-based authentication for the cache server

use super::config::AuthConfig;
use crate::cache::remote::protocol::AuthHeader;
use std::collections::HashSet;

/// Token validator for authentication
pub struct TokenValidator {
    /// Set of valid tokens for O(1) lookup
    valid_tokens: HashSet<String>,
    /// Whether authentication is required
    require_auth: bool,
}

impl TokenValidator {
    /// Create a new token validator from config
    pub fn new(config: &AuthConfig) -> Self {
        Self {
            valid_tokens: config.tokens.iter().cloned().collect(),
            require_auth: config.require_auth,
        }
    }

    /// Validate an authentication header
    pub fn validate(&self, auth: &AuthHeader) -> AuthResult {
        // If no auth required and no tokens configured, allow all
        if !self.require_auth && self.valid_tokens.is_empty() {
            return AuthResult::Allowed;
        }

        // Check if token is valid
        if self.valid_tokens.contains(&auth.token) {
            AuthResult::Allowed
        } else if auth.token.is_empty() {
            AuthResult::NoToken
        } else {
            AuthResult::InvalidToken
        }
    }

    /// Check if authentication is required
    pub fn is_required(&self) -> bool {
        self.require_auth
    }

    /// Get number of configured tokens
    pub fn token_count(&self) -> usize {
        self.valid_tokens.len()
    }
}

/// Result of authentication validation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthResult {
    /// Request is allowed
    Allowed,
    /// No token provided
    NoToken,
    /// Invalid token provided
    InvalidToken,
}

impl AuthResult {
    pub fn is_allowed(&self) -> bool {
        matches!(self, AuthResult::Allowed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_token() {
        let config = AuthConfig {
            tokens: vec!["secret123".to_string()],
            require_auth: true,
        };
        let validator = TokenValidator::new(&config);

        let auth = AuthHeader {
            token: "secret123".to_string(),
            client_id: None,
        };
        assert_eq!(validator.validate(&auth), AuthResult::Allowed);
    }

    #[test]
    fn test_invalid_token() {
        let config = AuthConfig {
            tokens: vec!["secret123".to_string()],
            require_auth: true,
        };
        let validator = TokenValidator::new(&config);

        let auth = AuthHeader {
            token: "wrongtoken".to_string(),
            client_id: None,
        };
        assert_eq!(validator.validate(&auth), AuthResult::InvalidToken);
    }

    #[test]
    fn test_no_token() {
        let config = AuthConfig {
            tokens: vec!["secret123".to_string()],
            require_auth: true,
        };
        let validator = TokenValidator::new(&config);

        let auth = AuthHeader {
            token: String::new(),
            client_id: None,
        };
        assert_eq!(validator.validate(&auth), AuthResult::NoToken);
    }

    #[test]
    fn test_auth_not_required() {
        let config = AuthConfig {
            tokens: vec![],
            require_auth: false,
        };
        let validator = TokenValidator::new(&config);

        let auth = AuthHeader {
            token: String::new(),
            client_id: None,
        };
        assert_eq!(validator.validate(&auth), AuthResult::Allowed);
    }
}
