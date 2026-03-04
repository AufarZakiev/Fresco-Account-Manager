use std::sync::Arc;
use std::time::Duration;

use quick_xml::events::Event;
use quick_xml::Reader;

use crate::mock_projects::MockProjectStore;

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
/// When `mock_store` is set, all calls are intercepted and return mock data.
pub struct ProjectRpcClient {
    http: reqwest::Client,
    mock_store: Option<Arc<MockProjectStore>>,
}

impl ProjectRpcClient {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");
        Self { http, mock_store: None }
    }

    pub fn with_mock(mock_store: Arc<MockProjectStore>) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build HTTP client");
        Self { http, mock_store: Some(mock_store) }
    }

    pub fn is_mock(&self) -> bool {
        self.mock_store.is_some()
    }

    /// Create a new account on a BOINC project. Returns the project authenticator.
    pub async fn create_account(
        &self,
        project_url: &str,
        email: &str,
        passwd_hash: &str,
        user_name: &str,
    ) -> Result<String, ProjectRpcError> {
        if let Some(ref mock) = self.mock_store {
            let xml = mock.build_create_account_xml(project_url, email, passwd_hash, user_name);
            return parse_authenticator_response(&xml);
        }

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
        if let Some(ref mock) = self.mock_store {
            let xml = mock.build_lookup_account_xml(project_url, email, passwd_hash);
            return parse_authenticator_response(&xml);
        }

        let url = format!(
            "{}lookup_account.php?email_addr={}&passwd_hash={}",
            project_url,
            urlencoding::encode(email),
            urlencoding::encode(passwd_hash),
        );

        let body = self.http.get(&url).send().await?.text().await?;
        parse_authenticator_response(&body)
    }

    /// Get account info from a BOINC project (including project_prefs).
    pub async fn get_info(
        &self,
        project_url: &str,
        account_key: &str,
    ) -> Result<AccountInfo, ProjectRpcError> {
        if let Some(ref mock) = self.mock_store {
            let xml = mock.build_get_info_xml(project_url, account_key);
            return parse_get_info_response(&xml);
        }

        let url = format!(
            "{}am_get_info.php?account_key={}",
            project_url,
            urlencoding::encode(account_key),
        );

        let body = self.http.get(&url).send().await?.text().await?;
        parse_get_info_response(&body)
    }

    /// Set account info on a BOINC project (including project_prefs).
    pub async fn set_info(
        &self,
        project_url: &str,
        account_key: &str,
        params: &SetInfoParams,
    ) -> Result<(), ProjectRpcError> {
        if let Some(ref mock) = self.mock_store {
            let xml = mock.build_set_info_xml(
                project_url,
                account_key,
                params.project_prefs.as_deref(),
            );
            if let Some(code) = parse_error_num(&xml) {
                if code != 0 {
                    return Err(ProjectRpcError::ProjectError {
                        code,
                        message: format!("am_set_info failed with code {code}"),
                    });
                }
            }
            return Ok(());
        }

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
        if let Some(ref prefs) = params.project_prefs {
            url.push_str(&format!("&project_prefs={}", urlencoding::encode(prefs)));
        }

        let body = self.http.get(&url).send().await?.text().await?;
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

    /// Get project configuration (apps, platforms, web_rpc_url_base).
    pub async fn get_project_config(
        &self,
        project_url: &str,
    ) -> Result<ProjectConfig, ProjectRpcError> {
        if self.mock_store.is_some() {
            let xml = MockProjectStore::build_project_config_xml(project_url);
            return parse_project_config_response(&xml);
        }

        let url = format!("{}get_project_config.php", project_url);
        let body = self.http.get(&url).send().await?.text().await?;
        parse_project_config_response(&body)
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
    pub project_prefs: Option<String>,
}

/// Parameters for am_set_info.php.
#[derive(Debug, Default)]
pub struct SetInfoParams {
    pub name: Option<String>,
    pub url: Option<String>,
    pub project_prefs: Option<String>,
}

/// Project configuration returned by get_project_config.php.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ProjectConfig {
    pub name: String,
    pub web_rpc_url_base: String,
    pub client_account_creation_disabled: bool,
    pub apps: Vec<ProjectConfigApp>,
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct ProjectConfigApp {
    pub name: String,
    pub id: i64,
    pub user_friendly_name: String,
}

fn parse_authenticator_response(xml: &str) -> Result<String, ProjectRpcError> {
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

    let project_prefs = extract_inner_xml(xml, "project_preferences")
        .or_else(|| extract_inner_xml(xml, "project_prefs"));

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
        project_prefs,
    })
}

fn parse_project_config_response(xml: &str) -> Result<ProjectConfig, ProjectRpcError> {
    if let Some(code) = parse_error_num(xml) {
        if code != 0 {
            return Err(ProjectRpcError::ProjectError {
                code,
                message: format!("get_project_config failed with code {code}"),
            });
        }
    }

    let mut config = ProjectConfig {
        name: parse_tag(xml, "name").unwrap_or_default(),
        web_rpc_url_base: parse_tag(xml, "web_rpc_url_base").unwrap_or_default(),
        ..Default::default()
    };

    // Check for client_account_creation_disabled (self-closing tag)
    config.client_account_creation_disabled = xml.contains("<client_account_creation_disabled");

    // Parse apps and platforms
    let mut reader = Reader::from_str(xml);
    let mut in_app = false;
    let mut current_app = ProjectConfigApp::default();
    let mut current_tag = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "app" {
                    in_app = true;
                    current_app = ProjectConfigApp::default();
                }
                current_tag = tag;
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().unwrap_or_default().trim().to_string();
                if in_app {
                    match current_tag.as_str() {
                        "name" => current_app.name = text,
                        "id" => current_app.id = text.parse().unwrap_or(0),
                        "user_friendly_name" => current_app.user_friendly_name = text,
                        _ => {}
                    }
                } else if current_tag == "platform" {
                    config.platforms.push(text);
                }
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag == "app" {
                    in_app = false;
                    config.apps.push(current_app.clone());
                }
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
    }

    Ok(config)
}

/// Extract the inner XML content of a tag (everything between <tag> and </tag>).
fn extract_inner_xml(xml: &str, tag_name: &str) -> Option<String> {
    let open = format!("<{}", tag_name);
    let close = format!("</{}>", tag_name);
    let start_idx = xml.find(&open)?;
    // Find the end of the opening tag
    let after_open = xml[start_idx..].find('>')?;
    let content_start = start_idx + after_open + 1;
    let end_idx = xml[content_start..].find(&close)?;
    let inner = xml[content_start..content_start + end_idx].trim();
    if inner.is_empty() {
        None
    } else {
        Some(inner.to_string())
    }
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
    fn test_parse_get_info_with_project_prefs() {
        let xml = r#"<am_get_info_reply>
    <success/>
    <id>1</id>
    <name>Jane</name>
    <project_preferences>
        <allow_beta_work>0</allow_beta_work>
    </project_preferences>
</am_get_info_reply>"#;
        let info = parse_get_info_response(xml).unwrap();
        assert!(info.project_prefs.is_some());
        assert!(info.project_prefs.unwrap().contains("allow_beta_work"));
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
        assert!(info.project_prefs.is_none());
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

    #[test]
    fn test_parse_project_config() {
        let xml = r#"<project_config>
    <name>Test Project</name>
    <web_rpc_url_base>https://test.example.com/</web_rpc_url_base>
    <client_account_creation_disabled/>
    <app>
        <name>test_app</name>
        <id>1</id>
        <user_friendly_name>Test Application</user_friendly_name>
    </app>
    <platform>windows_x86_64</platform>
    <platform>x86_64-pc-linux-gnu</platform>
</project_config>"#;
        let config = parse_project_config_response(xml).unwrap();
        assert_eq!(config.name, "Test Project");
        assert_eq!(config.web_rpc_url_base, "https://test.example.com/");
        assert!(config.client_account_creation_disabled);
        assert_eq!(config.apps.len(), 1);
        assert_eq!(config.apps[0].name, "test_app");
        assert_eq!(config.apps[0].id, 1);
        assert_eq!(config.platforms.len(), 2);
    }

    #[test]
    fn test_extract_inner_xml() {
        let xml = "<root><prefs><a>1</a><b>2</b></prefs></root>";
        let inner = extract_inner_xml(xml, "prefs").unwrap();
        assert_eq!(inner, "<a>1</a><b>2</b>");
    }
}
