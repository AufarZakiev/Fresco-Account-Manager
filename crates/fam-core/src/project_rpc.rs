use std::time::Duration;

use quick_xml::events::Event;
use quick_xml::Reader;

#[derive(Debug, thiserror::Error)]
pub enum ProjectRpcError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("XML parse error: {0}")]
    Xml(String),
    #[error("project error {code}: {message}")]
    ProjectError { code: i32, message: String },
}

/// BOINC project error codes.
pub const ERR_NOT_FOUND: i32 = -136;
pub const ERR_NON_UNIQUE: i32 = -137;
pub const ERR_PROJECT_DOWN: i32 = -183;
pub const ERR_ACCT_CREATION_DISABLED: i32 = -208;

/// Client for BOINC project Web RPC endpoints.
pub struct ProjectRpcClient {
    http: reqwest::Client,
}

impl ProjectRpcClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");
        Self { http }
    }

    /// Create a new account on a BOINC project. Returns the project authenticator.
    pub async fn create_account(
        &self,
        project_url: &str,
        email: &str,
        passwd_hash: &str,
        user_name: &str,
    ) -> Result<String, ProjectRpcError> {
        let url = format!(
            "{}create_account.php?email_addr={}&passwd_hash={}&user_name={}",
            project_url,
            urlencoding::encode(email),
            urlencoding::encode(passwd_hash),
            urlencoding::encode(user_name),
        );

        let body = self.http.get(&url).send().await?.text().await?;
        parse_authenticator_response(&body)
    }

    /// Look up an existing account on a BOINC project. Returns the project authenticator.
    pub async fn lookup_account(
        &self,
        project_url: &str,
        email: &str,
        passwd_hash: &str,
    ) -> Result<String, ProjectRpcError> {
        let url = format!(
            "{}lookup_account.php?email_addr={}&passwd_hash={}",
            project_url,
            urlencoding::encode(email),
            urlencoding::encode(passwd_hash),
        );

        let body = self.http.get(&url).send().await?.text().await?;
        parse_authenticator_response(&body)
    }

    /// Get account info from a BOINC project.
    pub async fn get_info(
        &self,
        project_url: &str,
        account_key: &str,
    ) -> Result<AccountInfo, ProjectRpcError> {
        let url = format!(
            "{}am_get_info.php?account_key={}",
            project_url,
            urlencoding::encode(account_key),
        );

        let body = self.http.get(&url).send().await?.text().await?;
        parse_get_info_response(&body)
    }

    /// Set account info on a BOINC project.
    pub async fn set_info(
        &self,
        project_url: &str,
        account_key: &str,
        params: &SetInfoParams,
    ) -> Result<(), ProjectRpcError> {
        let mut url = format!(
            "{}am_set_info.php?account_key={}",
            project_url,
            urlencoding::encode(account_key),
        );
        if let Some(ref name) = params.name {
            url.push_str(&format!("&name={}", urlencoding::encode(name)));
        }
        if let Some(ref url_str) = params.url {
            url.push_str(&format!("&url={}", urlencoding::encode(url_str)));
        }

        let body = self.http.get(&url).send().await?.text().await?;
        // Check for error
        if let Some(code) = parse_error_num(&body) {
            if code != 0 {
                return Err(ProjectRpcError::ProjectError {
                    code,
                    message: format!("am_set_info failed with code {code}"),
                });
            }
        }
        Ok(())
    }
}

impl Default for ProjectRpcClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Account info returned by am_get_info.php.
#[derive(Debug, Clone, Default)]
pub struct AccountInfo {
    pub id: i64,
    pub name: String,
    pub country: String,
    pub weak_auth: String,
    pub teamid: i64,
}

/// Parameters for am_set_info.php.
#[derive(Debug, Default)]
pub struct SetInfoParams {
    pub name: Option<String>,
    pub url: Option<String>,
}

fn parse_authenticator_response(xml: &str) -> Result<String, ProjectRpcError> {
    // Check for error first
    if let Some(code) = parse_error_num(xml) {
        if code != 0 {
            let msg = parse_tag(xml, "message").unwrap_or_default();
            return Err(ProjectRpcError::ProjectError { code, message: msg });
        }
    }

    parse_tag(xml, "authenticator")
        .ok_or_else(|| ProjectRpcError::Xml("missing <authenticator> in response".to_string()))
}

fn parse_get_info_response(xml: &str) -> Result<AccountInfo, ProjectRpcError> {
    if let Some(code) = parse_error_num(xml) {
        if code != 0 {
            return Err(ProjectRpcError::ProjectError {
                code,
                message: format!("am_get_info failed with code {code}"),
            });
        }
    }

    Ok(AccountInfo {
        id: parse_tag(xml, "id")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        name: parse_tag(xml, "name").unwrap_or_default(),
        country: parse_tag(xml, "country").unwrap_or_default(),
        weak_auth: parse_tag(xml, "weak_auth").unwrap_or_default(),
        teamid: parse_tag(xml, "teamid")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
    })
}

/// Quick helper to extract text content of a single XML tag.
fn parse_tag(xml: &str, tag_name: &str) -> Option<String> {
    let mut reader = Reader::from_str(xml);
    let mut current_tag = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
            }
            Ok(Event::Text(ref e)) => {
                if current_tag == tag_name {
                    return Some(e.unescape().unwrap_or_default().trim().to_string());
                }
            }
            Ok(Event::End(_)) => current_tag.clear(),
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }
    None
}

fn parse_error_num(xml: &str) -> Option<i32> {
    parse_tag(xml, "error_num").and_then(|s| s.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_create_account_success() {
        let xml = r#"<account_out>
    <authenticator>abc123def456</authenticator>
</account_out>"#;
        let auth = parse_authenticator_response(xml).unwrap();
        assert_eq!(auth, "abc123def456");
    }

    #[test]
    fn test_parse_create_account_error() {
        let xml = r#"<error>
    <error_num>-208</error_num>
    <message>Account creation disabled</message>
</error>"#;
        let err = parse_authenticator_response(xml).unwrap_err();
        match err {
            ProjectRpcError::ProjectError { code, .. } => assert_eq!(code, -208),
            _ => panic!("expected ProjectError"),
        }
    }

    #[test]
    fn test_parse_get_info() {
        let xml = r#"<am_get_info_reply>
    <success/>
    <id>12345</id>
    <name>John Doe</name>
    <country>US</country>
    <weak_auth>abc_123</weak_auth>
    <teamid>42</teamid>
</am_get_info_reply>"#;
        let info = parse_get_info_response(xml).unwrap();
        assert_eq!(info.id, 12345);
        assert_eq!(info.name, "John Doe");
        assert_eq!(info.teamid, 42);
    }

    #[test]
    fn test_parse_get_info_error() {
        let xml = r#"<am_get_info_reply>
    <error_num>-136</error_num>
</am_get_info_reply>"#;
        let err = parse_get_info_response(xml).unwrap_err();
        match err {
            ProjectRpcError::ProjectError { code, .. } => assert_eq!(code, -136),
            _ => panic!("expected ProjectError"),
        }
    }

    #[test]
    fn test_parse_get_info_minimal() {
        // Missing optional fields should default
        let xml = r#"<am_get_info_reply>
    <success/>
    <id>1</id>
    <name>Jane</name>
</am_get_info_reply>"#;
        let info = parse_get_info_response(xml).unwrap();
        assert_eq!(info.id, 1);
        assert_eq!(info.name, "Jane");
        assert_eq!(info.country, "");
        assert_eq!(info.weak_auth, "");
        assert_eq!(info.teamid, 0);
    }

    #[test]
    fn test_parse_lookup_account_not_found() {
        let xml = r#"<error>
    <error_num>-136</error_num>
    <message>Not found</message>
</error>"#;
        let err = parse_authenticator_response(xml).unwrap_err();
        match err {
            ProjectRpcError::ProjectError { code, message } => {
                assert_eq!(code, ERR_NOT_FOUND);
                assert_eq!(message, "Not found");
            }
            _ => panic!("expected ProjectError"),
        }
    }

    #[test]
    fn test_parse_non_unique_error() {
        let xml = r#"<error>
    <error_num>-137</error_num>
    <message>Already exists</message>
</error>"#;
        let err = parse_authenticator_response(xml).unwrap_err();
        match err {
            ProjectRpcError::ProjectError { code, .. } => {
                assert_eq!(code, ERR_NON_UNIQUE);
            }
            _ => panic!("expected ProjectError"),
        }
    }

    #[test]
    fn test_parse_authenticator_missing() {
        // Response with error_num=0 but no authenticator
        let xml = r#"<account_out>
    <error_num>0</error_num>
</account_out>"#;
        let err = parse_authenticator_response(xml).unwrap_err();
        match err {
            ProjectRpcError::Xml(msg) => {
                assert!(msg.contains("authenticator"));
            }
            _ => panic!("expected Xml error"),
        }
    }

    #[test]
    fn test_parse_tag_helper() {
        let xml = r#"<root><foo>bar</foo><baz>42</baz></root>"#;
        assert_eq!(parse_tag(xml, "foo"), Some("bar".to_string()));
        assert_eq!(parse_tag(xml, "baz"), Some("42".to_string()));
        assert_eq!(parse_tag(xml, "missing"), None);
    }

    #[test]
    fn test_parse_error_num_helper() {
        assert_eq!(parse_error_num("<r><error_num>-208</error_num></r>"), Some(-208));
        assert_eq!(parse_error_num("<r><error_num>0</error_num></r>"), Some(0));
        assert_eq!(parse_error_num("<r><other>1</other></r>"), None);
    }
}
