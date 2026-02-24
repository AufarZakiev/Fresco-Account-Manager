/// Build the `<project_config>` XML response that identifies this server
/// as a BOINC Account Manager.
pub fn build_project_config(name: &str, min_passwd_length: u32) -> String {
    format!(
        "<project_config>\n\
         \x20   <name>{name}</name>\n\
         \x20   <account_manager/>\n\
         \x20   <min_passwd_length>{min_passwd_length}</min_passwd_length>\n\
         </project_config>\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_project_config() {
        let xml = build_project_config("Fresco Account Manager", 6);
        assert!(xml.contains("<account_manager/>"));
        assert!(xml.contains("<name>Fresco Account Manager</name>"));
        assert!(xml.contains("<min_passwd_length>6</min_passwd_length>"));
    }

    #[test]
    fn test_build_project_config_is_valid_xml() {
        let xml = build_project_config("Test", 8);
        assert!(xml.starts_with("<project_config>"));
        assert!(xml.contains("</project_config>"));
    }

    #[test]
    fn test_build_project_config_custom_values() {
        let xml = build_project_config("My Custom AM", 12);
        assert!(xml.contains("<name>My Custom AM</name>"));
        assert!(xml.contains("<min_passwd_length>12</min_passwd_length>"));
        assert!(xml.contains("<account_manager/>"));
    }
}
