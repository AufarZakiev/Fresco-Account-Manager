pub mod admin;
pub mod auth;
pub mod hosts;
pub mod preferences;
pub mod projects;
pub mod stats;
pub mod user_projects;

/// Extract session ID from the Cookie header.
pub fn session_from_cookie(headers: &axum::http::HeaderMap) -> Option<String> {
    let cookie_header = headers.get("cookie")?.to_str().ok()?;
    for part in cookie_header.split(';') {
        let part = part.trim();
        if let Some(value) = part.strip_prefix("session=") {
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;

    #[test]
    fn test_session_from_cookie_basic() {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", "session=abc123".parse().unwrap());
        assert_eq!(session_from_cookie(&headers), Some("abc123".to_string()));
    }

    #[test]
    fn test_session_from_cookie_multiple_cookies() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            "theme=dark; session=my-session-id; lang=en".parse().unwrap(),
        );
        assert_eq!(
            session_from_cookie(&headers),
            Some("my-session-id".to_string())
        );
    }

    #[test]
    fn test_session_from_cookie_missing() {
        let headers = HeaderMap::new();
        assert_eq!(session_from_cookie(&headers), None);
    }

    #[test]
    fn test_session_from_cookie_no_session_key() {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", "theme=dark; lang=en".parse().unwrap());
        assert_eq!(session_from_cookie(&headers), None);
    }

    #[test]
    fn test_session_from_cookie_empty_value() {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", "session=".parse().unwrap());
        assert_eq!(session_from_cookie(&headers), None);
    }

    #[test]
    fn test_session_from_cookie_with_spaces() {
        let mut headers = HeaderMap::new();
        headers.insert("cookie", "  session=spaced  ; other=val".parse().unwrap());
        assert_eq!(session_from_cookie(&headers), Some("spaced".to_string()));
    }

    #[test]
    fn test_session_from_cookie_uuid() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "cookie",
            "session=550e8400-e29b-41d4-a716-446655440000"
                .parse()
                .unwrap(),
        );
        assert_eq!(
            session_from_cookie(&headers),
            Some("550e8400-e29b-41d4-a716-446655440000".to_string())
        );
    }
}
