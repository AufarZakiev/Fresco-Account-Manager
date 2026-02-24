/// A project account entry in the AM reply.
#[derive(Debug, Clone)]
pub struct AccountEntry {
    pub url: String,
    pub url_signature: String,
    pub authenticator: String,
    pub resource_share: f32,
    pub suspend: bool,
    pub dont_request_more_work: bool,
    pub detach_when_done: bool,
    pub detach: bool,
    pub update: bool,
    pub no_rsc: Vec<String>,
    pub abort_not_started: bool,
}

/// Parameters for building an `<acct_mgr_reply>`.
#[derive(Debug)]
pub struct AcctMgrReply {
    pub name: String,
    pub error_num: i32,
    pub error_msg: Option<String>,
    pub authenticator: Option<String>,
    pub user_name: Option<String>,
    pub team_name: Option<String>,
    pub signing_key: String,
    pub repeat_sec: u32,
    pub accounts: Vec<AccountEntry>,
    pub global_preferences: Option<String>,
    pub host_venue: Option<String>,
    pub opaque: Option<String>,
    pub messages: Vec<String>,
}

impl Default for AcctMgrReply {
    fn default() -> Self {
        Self {
            name: String::new(),
            error_num: 0,
            error_msg: None,
            authenticator: None,
            user_name: None,
            team_name: None,
            signing_key: String::new(),
            repeat_sec: 86400,
            accounts: Vec::new(),
            global_preferences: None,
            host_venue: None,
            opaque: None,
            messages: Vec::new(),
        }
    }
}

/// Build an error reply.
pub fn build_error_reply(error_num: i32, error_msg: &str) -> String {
    format!(
        "<acct_mgr_reply>\n\
         \x20   <error_num>{error_num}</error_num>\n\
         \x20   <error_msg>{}</error_msg>\n\
         </acct_mgr_reply>\n",
        xml_escape(error_msg)
    )
}

/// Build a full `<acct_mgr_reply>` XML string.
pub fn build_acct_mgr_reply(reply: &AcctMgrReply) -> String {
    let mut xml = String::from("<acct_mgr_reply>\n");

    if reply.error_num != 0 {
        xml.push_str(&format!("   <error_num>{}</error_num>\n", reply.error_num));
        if let Some(ref msg) = reply.error_msg {
            xml.push_str(&format!("   <error_msg>{}</error_msg>\n", xml_escape(msg)));
        }
        xml.push_str("</acct_mgr_reply>\n");
        return xml;
    }

    xml.push_str(&format!("   <name>{}</name>\n", xml_escape(&reply.name)));

    if let Some(ref auth) = reply.authenticator {
        xml.push_str(&format!("   <authenticator>{auth}</authenticator>\n"));
    }
    if let Some(ref name) = reply.user_name {
        xml.push_str(&format!("   <user_name>{}</user_name>\n", xml_escape(name)));
    }
    if let Some(ref team) = reply.team_name {
        xml.push_str(&format!("   <team_name>{}</team_name>\n", xml_escape(team)));
    }

    xml.push_str(&format!(
        "   <signing_key>\n{}</signing_key>\n",
        reply.signing_key
    ));
    xml.push_str(&format!(
        "   <repeat_sec>{}</repeat_sec>\n",
        reply.repeat_sec
    ));

    for msg in &reply.messages {
        xml.push_str(&format!("   <message>{}</message>\n", xml_escape(msg)));
    }

    if let Some(ref prefs) = reply.global_preferences {
        xml.push_str(&format!(
            "   <global_preferences>\n{prefs}\n   </global_preferences>\n"
        ));
    }

    if let Some(ref venue) = reply.host_venue {
        xml.push_str(&format!("   <host_venue>{venue}</host_venue>\n"));
    }

    if let Some(ref opaque) = reply.opaque {
        xml.push_str(&format!("   <opaque>\n{opaque}\n   </opaque>\n"));
    }

    for account in &reply.accounts {
        xml.push_str("   <account>\n");
        xml.push_str(&format!("      <url>{}</url>\n", xml_escape(&account.url)));
        xml.push_str(&format!(
            "      <url_signature>\n{}</url_signature>\n",
            account.url_signature
        ));
        xml.push_str(&format!(
            "      <authenticator>{}</authenticator>\n",
            account.authenticator
        ));
        if account.detach {
            xml.push_str("      <detach>1</detach>\n");
        }
        if account.update {
            xml.push_str("      <update>1</update>\n");
        }
        if account.suspend {
            xml.push_str("      <suspend>1</suspend>\n");
        }
        if account.abort_not_started {
            xml.push_str("      <abort_not_started>1</abort_not_started>\n");
        }
        if account.dont_request_more_work {
            xml.push_str("      <dont_request_more_work>1</dont_request_more_work>\n");
        }
        if account.detach_when_done {
            xml.push_str("      <detach_when_done>1</detach_when_done>\n");
        }
        xml.push_str(&format!(
            "      <resource_share>{}</resource_share>\n",
            account.resource_share
        ));
        for rsc in &account.no_rsc {
            xml.push_str(&format!("      <no_rsc>{rsc}</no_rsc>\n"));
        }
        xml.push_str("   </account>\n");
    }

    xml.push_str("</acct_mgr_reply>\n");
    xml
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_reply() {
        let xml = build_error_reply(-206, "Invalid password");
        assert!(xml.contains("<error_num>-206</error_num>"));
        assert!(xml.contains("<error_msg>Invalid password</error_msg>"));
    }

    #[test]
    fn test_success_reply() {
        let reply = AcctMgrReply {
            name: "Fresco Account Manager".to_string(),
            authenticator: Some("auth-token-123".to_string()),
            user_name: Some("John Doe".to_string()),
            signing_key: "1024\nabcdef\n.\n".to_string(),
            repeat_sec: 86400,
            accounts: vec![AccountEntry {
                url: "https://einsteinathome.org/".to_string(),
                url_signature: "aabbcc\n.\n".to_string(),
                authenticator: "project-auth-456".to_string(),
                resource_share: 100.0,
                suspend: false,
                dont_request_more_work: false,
                detach_when_done: false,
                detach: false,
                update: false,
                no_rsc: vec![],
                abort_not_started: false,
            }],
            ..Default::default()
        };

        let xml = build_acct_mgr_reply(&reply);
        assert!(xml.contains("<name>Fresco Account Manager</name>"));
        assert!(xml.contains("<authenticator>auth-token-123</authenticator>"));
        assert!(xml.contains("<signing_key>"));
        assert!(xml.contains("<url>https://einsteinathome.org/</url>"));
        assert!(xml.contains("<authenticator>project-auth-456</authenticator>"));
    }

    #[test]
    fn test_reply_with_control_flags() {
        let reply = AcctMgrReply {
            name: "FAM".to_string(),
            signing_key: "key".to_string(),
            accounts: vec![AccountEntry {
                url: "https://example.com/".to_string(),
                url_signature: "sig".to_string(),
                authenticator: "auth".to_string(),
                resource_share: 50.0,
                suspend: true,
                dont_request_more_work: true,
                detach_when_done: true,
                detach: false,
                update: true,
                no_rsc: vec!["GPU".to_string(), "CPU".to_string()],
                abort_not_started: true,
            }],
            ..Default::default()
        };

        let xml = build_acct_mgr_reply(&reply);
        assert!(xml.contains("<suspend>1</suspend>"));
        assert!(xml.contains("<dont_request_more_work>1</dont_request_more_work>"));
        assert!(xml.contains("<detach_when_done>1</detach_when_done>"));
        assert!(xml.contains("<update>1</update>"));
        assert!(xml.contains("<abort_not_started>1</abort_not_started>"));
        assert!(xml.contains("<no_rsc>GPU</no_rsc>"));
        assert!(xml.contains("<no_rsc>CPU</no_rsc>"));
        assert!(xml.contains("<resource_share>50</resource_share>"));
        // detach should NOT be present since it's false
        assert!(!xml.contains("<detach>1</detach>"));
    }

    #[test]
    fn test_reply_with_detach() {
        let reply = AcctMgrReply {
            name: "FAM".to_string(),
            signing_key: "key".to_string(),
            accounts: vec![AccountEntry {
                url: "https://example.com/".to_string(),
                url_signature: "sig".to_string(),
                authenticator: "auth".to_string(),
                resource_share: 100.0,
                suspend: false,
                dont_request_more_work: false,
                detach_when_done: false,
                detach: true,
                update: false,
                no_rsc: vec![],
                abort_not_started: false,
            }],
            ..Default::default()
        };

        let xml = build_acct_mgr_reply(&reply);
        assert!(xml.contains("<detach>1</detach>"));
    }

    #[test]
    fn test_reply_with_messages() {
        let reply = AcctMgrReply {
            name: "FAM".to_string(),
            signing_key: "key".to_string(),
            messages: vec![
                "Welcome to FAM!".to_string(),
                "Maintenance scheduled for Sunday".to_string(),
            ],
            ..Default::default()
        };

        let xml = build_acct_mgr_reply(&reply);
        assert!(xml.contains("<message>Welcome to FAM!</message>"));
        assert!(xml.contains("<message>Maintenance scheduled for Sunday</message>"));
    }

    #[test]
    fn test_reply_with_preferences_and_venue() {
        let reply = AcctMgrReply {
            name: "FAM".to_string(),
            signing_key: "key".to_string(),
            global_preferences: Some("<run_on_batteries>0</run_on_batteries>".to_string()),
            host_venue: Some("home".to_string()),
            opaque: Some("<fam_host_id>42</fam_host_id>".to_string()),
            ..Default::default()
        };

        let xml = build_acct_mgr_reply(&reply);
        assert!(xml.contains("<global_preferences>"));
        assert!(xml.contains("<run_on_batteries>0</run_on_batteries>"));
        assert!(xml.contains("</global_preferences>"));
        assert!(xml.contains("<host_venue>home</host_venue>"));
        assert!(xml.contains("<opaque>"));
        assert!(xml.contains("<fam_host_id>42</fam_host_id>"));
    }

    #[test]
    fn test_reply_xml_escaping() {
        let reply = AcctMgrReply {
            name: "FAM & Friends <Test>".to_string(),
            authenticator: Some("auth".to_string()),
            user_name: Some("O'Brien \"Bob\"".to_string()),
            signing_key: "key".to_string(),
            ..Default::default()
        };

        let xml = build_acct_mgr_reply(&reply);
        assert!(xml.contains("<name>FAM &amp; Friends &lt;Test&gt;</name>"));
        assert!(xml.contains("<user_name>O&apos;Brien &quot;Bob&quot;</user_name>"));
    }

    #[test]
    fn test_reply_with_team_name() {
        let reply = AcctMgrReply {
            name: "FAM".to_string(),
            signing_key: "key".to_string(),
            team_name: Some("BOINC Team Alpha".to_string()),
            ..Default::default()
        };

        let xml = build_acct_mgr_reply(&reply);
        assert!(xml.contains("<team_name>BOINC Team Alpha</team_name>"));
    }

    #[test]
    fn test_reply_multiple_accounts() {
        let reply = AcctMgrReply {
            name: "FAM".to_string(),
            signing_key: "key".to_string(),
            accounts: vec![
                AccountEntry {
                    url: "https://project1.com/".to_string(),
                    url_signature: "sig1".to_string(),
                    authenticator: "auth1".to_string(),
                    resource_share: 200.0,
                    suspend: false,
                    dont_request_more_work: false,
                    detach_when_done: false,
                    detach: false,
                    update: false,
                    no_rsc: vec![],
                    abort_not_started: false,
                },
                AccountEntry {
                    url: "https://project2.com/".to_string(),
                    url_signature: "sig2".to_string(),
                    authenticator: "auth2".to_string(),
                    resource_share: 50.0,
                    suspend: true,
                    dont_request_more_work: false,
                    detach_when_done: false,
                    detach: false,
                    update: false,
                    no_rsc: vec![],
                    abort_not_started: false,
                },
            ],
            ..Default::default()
        };

        let xml = build_acct_mgr_reply(&reply);
        // Count <account> tags
        let account_count = xml.matches("<account>").count();
        assert_eq!(account_count, 2);
        assert!(xml.contains("auth1"));
        assert!(xml.contains("auth2"));
        assert!(xml.contains("<resource_share>200</resource_share>"));
        assert!(xml.contains("<resource_share>50</resource_share>"));
    }
}
