//! Configuration validation logic.

use crate::config::config::Config;
use crate::error::{Error, Result};
use regex::Regex;

/// Minimum length for authorization token.
const MIN_TOKEN_LENGTH: usize = 50;

/// Minimum length for user agent.
const MIN_USER_AGENT_LENGTH: usize = 40;

/// Minimum username length.
const MIN_USERNAME_LENGTH: usize = 4;

/// Maximum username length.
const MAX_USERNAME_LENGTH: usize = 30;

/// Validate the entire configuration.
pub fn validate_config(config: &Config) -> Result<()> {
    validate_token(&config.my_account.authorization_token)?;
    validate_user_agent(&config.my_account.user_agent)?;
    validate_check_key(&config.my_account.check_key)?;
    validate_usernames(&config.targeted_creator.usernames)?;

    Ok(())
}

/// Validate the authorization token.
pub fn validate_token(token: &str) -> Result<()> {
    if token.is_empty() {
        return Err(Error::MissingConfig("authorization_token".to_string()));
    }

    if token.len() < MIN_TOKEN_LENGTH {
        return Err(Error::ConfigValidation {
            field: "authorization_token".to_string(),
            message: format!(
                "Token must be at least {} characters (got {})",
                MIN_TOKEN_LENGTH,
                token.len()
            ),
        });
    }

    // Check for placeholder values
    let token_lower = token.to_lowercase();
    if token_lower.contains("replaceme") || token_lower.contains("your_token") {
        return Err(Error::ConfigValidation {
            field: "authorization_token".to_string(),
            message: "Token appears to be a placeholder. Please provide your actual Fansly token."
                .to_string(),
        });
    }

    Ok(())
}

/// Validate the user agent string.
pub fn validate_user_agent(user_agent: &str) -> Result<()> {
    if user_agent.is_empty() {
        return Err(Error::MissingConfig("user_agent".to_string()));
    }

    if user_agent.len() < MIN_USER_AGENT_LENGTH {
        return Err(Error::ConfigValidation {
            field: "user_agent".to_string(),
            message: format!(
                "User agent must be at least {} characters (got {})",
                MIN_USER_AGENT_LENGTH,
                user_agent.len()
            ),
        });
    }

    // Check for placeholder values
    let ua_lower = user_agent.to_lowercase();
    if ua_lower.contains("replaceme") || ua_lower.contains("your_user_agent") {
        return Err(Error::ConfigValidation {
            field: "user_agent".to_string(),
            message:
                "User agent appears to be a placeholder. Please provide your browser's user agent."
                    .to_string(),
        });
    }

    Ok(())
}

/// Validate the check key.
pub fn validate_check_key(check_key: &str) -> Result<()> {
    if check_key.is_empty() {
        return Err(Error::MissingConfig("check_key".to_string()));
    }

    Ok(())
}

/// Validate creator usernames.
pub fn validate_usernames<S: AsRef<str>, I: IntoIterator<Item = S>>(usernames: I) -> Result<()> {
    let usernames: Vec<_> = usernames.into_iter().collect();

    if usernames.is_empty() {
        return Err(Error::MissingConfig(
            "usernames (at least one creator username required)".to_string(),
        ));
    }

    // Username pattern: 4-30 chars, alphanumeric, hyphens, underscores
    let username_pattern = Regex::new(r"^[a-zA-Z0-9_-]{4,30}$").unwrap();

    for username in usernames {
        let username = username.as_ref();

        // Remove leading @ if present
        let clean_username = username.trim_start_matches('@');

        if clean_username.len() < MIN_USERNAME_LENGTH {
            return Err(Error::ConfigValidation {
                field: "usernames".to_string(),
                message: format!(
                    "Username '{}' is too short (minimum {} characters)",
                    username, MIN_USERNAME_LENGTH
                ),
            });
        }

        if clean_username.len() > MAX_USERNAME_LENGTH {
            return Err(Error::ConfigValidation {
                field: "usernames".to_string(),
                message: format!(
                    "Username '{}' is too long (maximum {} characters)",
                    username, MAX_USERNAME_LENGTH
                ),
            });
        }

        if !username_pattern.is_match(clean_username) {
            return Err(Error::ConfigValidation {
                field: "usernames".to_string(),
                message: format!(
                    "Username '{}' contains invalid characters. Only alphanumeric, hyphens, and underscores allowed.",
                    username
                ),
            });
        }

        // Check for placeholder values
        let lower = clean_username.to_lowercase();
        if lower == "replaceme" || lower == "username" || lower == "creator" {
            return Err(Error::ConfigValidation {
                field: "usernames".to_string(),
                message: format!(
                    "Username '{}' appears to be a placeholder. Please provide actual creator usernames.",
                    username
                ),
            });
        }
    }

    Ok(())
}

/// Extract post ID from a URL or direct ID string.
pub fn parse_post_id(input: &str) -> Result<String> {
    let input = input.trim();

    // If it's a URL, extract the post ID
    if input.starts_with("http://") || input.starts_with("https://") {
        // Pattern: https://fansly.com/post/1234567890123
        let post_pattern = Regex::new(r"/post/(\d{10,})").unwrap();

        if let Some(captures) = post_pattern.captures(input) {
            if let Some(id) = captures.get(1) {
                return Ok(id.as_str().to_string());
            }
        }

        return Err(Error::ConfigValidation {
            field: "post_id".to_string(),
            message: format!("Could not extract post ID from URL: {}", input),
        });
    }

    // Direct ID - must be 10+ digits
    let id_pattern = Regex::new(r"^\d{10,}$").unwrap();
    if id_pattern.is_match(input) {
        return Ok(input.to_string());
    }

    Err(Error::ConfigValidation {
        field: "post_id".to_string(),
        message: format!(
            "Invalid post ID: '{}'. Must be 10+ digits or a valid Fansly post URL.",
            input
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_username() {
        assert!(validate_usernames(&["valid_user123"]).is_ok());
        assert!(validate_usernames(&["user-name"]).is_ok());
        assert!(validate_usernames(&["UserName123"]).is_ok());
    }

    #[test]
    fn test_invalid_username_too_short() {
        assert!(validate_usernames(&["abc"]).is_err());
    }

    #[test]
    fn test_invalid_username_placeholder() {
        assert!(validate_usernames(&["replaceme"]).is_err());
    }

    #[test]
    fn test_parse_post_id_direct() {
        assert_eq!(parse_post_id("1234567890123").unwrap(), "1234567890123");
    }

    #[test]
    fn test_parse_post_id_url() {
        let url = "https://fansly.com/post/1234567890123";
        assert_eq!(parse_post_id(url).unwrap(), "1234567890123");
    }

    #[test]
    fn test_parse_post_id_invalid() {
        assert!(parse_post_id("12345").is_err()); // Too short
        assert!(parse_post_id("not-a-number").is_err());
    }
}
