use quick_xml::events::Event;
use quick_xml::Reader;

/// Parsed BOINC `<acct_mgr_request>` XML.
#[derive(Debug, Default)]
pub struct AcctMgrRequest {
    pub authenticator: Option<String>,
    pub name: Option<String>,
    pub password_hash: Option<String>,
    pub host_cpid: String,
    pub previous_host_cpid: Option<String>,
    pub domain_name: Option<String>,
    pub client_version: Option<String>,
    pub run_mode: Option<String>,
    pub platform_name: Option<String>,
    pub projects: Vec<RequestProject>,
    pub global_preferences: Option<String>,
    pub working_global_preferences: Option<String>,
    pub host_info: Option<String>,
    pub opaque: Option<String>,
}

/// Per-project status reported by the BOINC client.
#[derive(Debug, Default, Clone)]
pub struct RequestProject {
    pub url: String,
    pub project_name: String,
    pub suspended_via_gui: bool,
    pub hostid: i64,
    pub attached_via_acct_mgr: bool,
    pub dont_request_more_work: bool,
    pub detach_when_done: bool,
    pub ended: bool,
    pub resource_share: f64,
    pub account_key: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum XmlParseError {
    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::Error),
    #[error("missing required field: {0}")]
    MissingField(&'static str),
}

/// Parse an `<acct_mgr_request>` XML body into an `AcctMgrRequest`.
pub fn parse_acct_mgr_request(xml: &str) -> Result<AcctMgrRequest, XmlParseError> {
    let mut reader = Reader::from_str(xml);
    let mut req = AcctMgrRequest::default();
    let mut current_tag = String::new();
    let mut in_project = false;
    let mut current_project = RequestProject::default();
    let mut in_raw_section: Option<String> = None;
    let mut raw_depth = 0;
    let mut raw_content = String::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                // Handle raw XML sections (store as opaque string)
                if in_raw_section.is_some() {
                    raw_depth += 1;
                    raw_content.push('<');
                    raw_content.push_str(&tag);
                    raw_content.push('>');
                    continue;
                }

                match tag.as_str() {
                    "project" => {
                        in_project = true;
                        current_project = RequestProject::default();
                    }
                    "global_preferences"
                    | "working_global_preferences"
                    | "host_info"
                    | "opaque" => {
                        in_raw_section = Some(tag.clone());
                        raw_depth = 0;
                        raw_content.clear();
                    }
                    _ => {}
                }
                current_tag = tag;
            }
            Ok(Event::End(ref e)) => {
                let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                if let Some(ref section) = in_raw_section {
                    if tag == *section && raw_depth == 0 {
                        match section.as_str() {
                            "global_preferences" => {
                                req.global_preferences = Some(raw_content.clone());
                            }
                            "working_global_preferences" => {
                                req.working_global_preferences = Some(raw_content.clone());
                            }
                            "host_info" => req.host_info = Some(raw_content.clone()),
                            "opaque" => req.opaque = Some(raw_content.clone()),
                            _ => {}
                        }
                        in_raw_section = None;
                    } else {
                        raw_depth -= 1;
                        raw_content.push_str("</");
                        raw_content.push_str(&tag);
                        raw_content.push('>');
                    }
                    continue;
                }

                if tag == "project" && in_project {
                    req.projects.push(current_project.clone());
                    in_project = false;
                }
                current_tag.clear();
            }
            Ok(Event::Text(ref e)) => {
                if in_raw_section.is_some() {
                    raw_content.push_str(&e.unescape().unwrap_or_default());
                    continue;
                }

                let text = e.unescape().unwrap_or_default().trim().to_string();
                if text.is_empty() {
                    continue;
                }

                if in_project {
                    match current_tag.as_str() {
                        "url" => current_project.url = text,
                        "project_name" => current_project.project_name = text,
                        "suspended_via_gui" => {
                            current_project.suspended_via_gui = text == "1";
                        }
                        "hostid" => current_project.hostid = text.parse().unwrap_or(0),
                        "attached_via_acct_mgr" => {
                            current_project.attached_via_acct_mgr = text == "1";
                        }
                        "dont_request_more_work" => {
                            current_project.dont_request_more_work = text == "1";
                        }
                        "detach_when_done" => {
                            current_project.detach_when_done = text == "1";
                        }
                        "ended" => current_project.ended = text == "1",
                        "resource_share" => {
                            current_project.resource_share = text.parse().unwrap_or(100.0);
                        }
                        "account_key" => current_project.account_key = Some(text),
                        _ => {}
                    }
                } else {
                    match current_tag.as_str() {
                        "authenticator" => req.authenticator = Some(text),
                        "name" => req.name = Some(text),
                        "password_hash" => req.password_hash = Some(text),
                        "host_cpid" => req.host_cpid = text,
                        "previous_host_cpid" => req.previous_host_cpid = Some(text),
                        "domain_name" => req.domain_name = Some(text),
                        "client_version" => req.client_version = Some(text),
                        "run_mode" => req.run_mode = Some(text),
                        "platform_name" => req.platform_name = Some(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                if in_raw_section.is_some() {
                    let tag = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    raw_content.push('<');
                    raw_content.push_str(&tag);
                    raw_content.push_str("/>");
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(XmlParseError::Xml(e)),
            _ => {}
        }
    }
    Ok(req)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_request() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<acct_mgr_request>
    <name>user@example.com</name>
    <password_hash>abc123def456</password_hash>
    <host_cpid>CPID_123</host_cpid>
    <domain_name>myhost</domain_name>
    <client_version>7.24.1</client_version>
    <platform_name>windows_x86_64</platform_name>
    <project>
        <url>https://einsteinathome.org/</url>
        <project_name>Einstein@Home</project_name>
        <attached_via_acct_mgr>1</attached_via_acct_mgr>
        <resource_share>100.0</resource_share>
        <account_key>AUTH_KEY_123</account_key>
    </project>
    <opaque>
        <fam_host_id>42</fam_host_id>
    </opaque>
</acct_mgr_request>"#;

        let req = parse_acct_mgr_request(xml).unwrap();
        assert_eq!(req.name.as_deref(), Some("user@example.com"));
        assert_eq!(req.password_hash.as_deref(), Some("abc123def456"));
        assert_eq!(req.host_cpid, "CPID_123");
        assert_eq!(req.projects.len(), 1);
        assert_eq!(req.projects[0].url, "https://einsteinathome.org/");
        assert!(req.projects[0].attached_via_acct_mgr);
        assert_eq!(req.projects[0].account_key.as_deref(), Some("AUTH_KEY_123"));
        assert!(req.opaque.is_some());
    }

    #[test]
    fn test_parse_authenticator_auth() {
        let xml = r#"<acct_mgr_request>
    <authenticator>my-auth-token-123</authenticator>
    <host_cpid>CPID_456</host_cpid>
</acct_mgr_request>"#;

        let req = parse_acct_mgr_request(xml).unwrap();
        assert_eq!(req.authenticator.as_deref(), Some("my-auth-token-123"));
        assert!(req.name.is_none());
    }

    #[test]
    fn test_parse_multiple_projects() {
        let xml = r#"<acct_mgr_request>
    <authenticator>token-abc</authenticator>
    <host_cpid>CPID_789</host_cpid>
    <project>
        <url>https://einsteinathome.org/</url>
        <project_name>Einstein@Home</project_name>
        <attached_via_acct_mgr>1</attached_via_acct_mgr>
        <suspended_via_gui>0</suspended_via_gui>
        <resource_share>200.0</resource_share>
    </project>
    <project>
        <url>https://www.worldcommunitygrid.org/</url>
        <project_name>World Community Grid</project_name>
        <attached_via_acct_mgr>1</attached_via_acct_mgr>
        <suspended_via_gui>1</suspended_via_gui>
        <dont_request_more_work>1</dont_request_more_work>
        <resource_share>50.0</resource_share>
    </project>
    <project>
        <url>https://boinc.bakerlab.org/rosetta/</url>
        <project_name>Rosetta@Home</project_name>
        <attached_via_acct_mgr>0</attached_via_acct_mgr>
        <ended>1</ended>
        <resource_share>100.0</resource_share>
    </project>
</acct_mgr_request>"#;

        let req = parse_acct_mgr_request(xml).unwrap();
        assert_eq!(req.projects.len(), 3);

        assert_eq!(req.projects[0].project_name, "Einstein@Home");
        assert!(!req.projects[0].suspended_via_gui);
        assert_eq!(req.projects[0].resource_share, 200.0);

        assert_eq!(req.projects[1].project_name, "World Community Grid");
        assert!(req.projects[1].suspended_via_gui);
        assert!(req.projects[1].dont_request_more_work);
        assert_eq!(req.projects[1].resource_share, 50.0);

        assert_eq!(req.projects[2].project_name, "Rosetta@Home");
        assert!(!req.projects[2].attached_via_acct_mgr);
        assert!(req.projects[2].ended);
    }

    #[test]
    fn test_parse_host_info_raw_xml() {
        let xml = r#"<acct_mgr_request>
    <authenticator>token</authenticator>
    <host_cpid>CPID</host_cpid>
    <host_info>
        <timezone>-28800</timezone>
        <domain_name>mypc</domain_name>
        <p_ncpus>8</p_ncpus>
        <p_vendor>GenuineIntel</p_vendor>
        <p_model>Intel Core i7</p_model>
        <m_nbytes>17179869184</m_nbytes>
    </host_info>
    <global_preferences>
        <mod_time>1700000000</mod_time>
        <run_on_batteries>0</run_on_batteries>
        <max_ncpus_pct>75.000000</max_ncpus_pct>
    </global_preferences>
</acct_mgr_request>"#;

        let req = parse_acct_mgr_request(xml).unwrap();

        // host_info should be captured as raw XML
        let hi = req.host_info.unwrap();
        assert!(hi.contains("<p_ncpus>8</p_ncpus>"));
        assert!(hi.contains("<p_vendor>GenuineIntel</p_vendor>"));

        // global_preferences should be captured as raw XML
        let gp = req.global_preferences.unwrap();
        assert!(gp.contains("<mod_time>1700000000</mod_time>"));
        assert!(gp.contains("<max_ncpus_pct>75.000000</max_ncpus_pct>"));
    }

    #[test]
    fn test_parse_empty_request_fields() {
        let xml = r#"<acct_mgr_request>
    <host_cpid>CPID_EMPTY</host_cpid>
</acct_mgr_request>"#;

        let req = parse_acct_mgr_request(xml).unwrap();
        assert_eq!(req.host_cpid, "CPID_EMPTY");
        assert!(req.authenticator.is_none());
        assert!(req.name.is_none());
        assert!(req.password_hash.is_none());
        assert!(req.domain_name.is_none());
        assert!(req.client_version.is_none());
        assert!(req.platform_name.is_none());
        assert!(req.run_mode.is_none());
        assert!(req.global_preferences.is_none());
        assert!(req.host_info.is_none());
        assert!(req.opaque.is_none());
        assert!(req.projects.is_empty());
    }

    #[test]
    fn test_parse_project_with_detach_when_done() {
        let xml = r#"<acct_mgr_request>
    <host_cpid>CPID</host_cpid>
    <project>
        <url>https://example.com/</url>
        <project_name>Test</project_name>
        <detach_when_done>1</detach_when_done>
        <hostid>42</hostid>
        <account_key>secret-key</account_key>
    </project>
</acct_mgr_request>"#;

        let req = parse_acct_mgr_request(xml).unwrap();
        assert_eq!(req.projects.len(), 1);
        assert!(req.projects[0].detach_when_done);
        assert_eq!(req.projects[0].hostid, 42);
        assert_eq!(req.projects[0].account_key.as_deref(), Some("secret-key"));
    }

    #[test]
    fn test_parse_request_with_all_fields() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<acct_mgr_request>
    <name>user@test.com</name>
    <password_hash>deadbeef12345678</password_hash>
    <host_cpid>CPID_FULL</host_cpid>
    <previous_host_cpid>CPID_OLD</previous_host_cpid>
    <domain_name>workstation</domain_name>
    <client_version>8.0.2</client_version>
    <run_mode>auto</run_mode>
    <platform_name>x86_64-pc-linux-gnu</platform_name>
</acct_mgr_request>"#;

        let req = parse_acct_mgr_request(xml).unwrap();
        assert_eq!(req.name.as_deref(), Some("user@test.com"));
        assert_eq!(req.password_hash.as_deref(), Some("deadbeef12345678"));
        assert_eq!(req.host_cpid, "CPID_FULL");
        assert_eq!(req.previous_host_cpid.as_deref(), Some("CPID_OLD"));
        assert_eq!(req.domain_name.as_deref(), Some("workstation"));
        assert_eq!(req.client_version.as_deref(), Some("8.0.2"));
        assert_eq!(req.run_mode.as_deref(), Some("auto"));
        assert_eq!(req.platform_name.as_deref(), Some("x86_64-pc-linux-gnu"));
    }
}
