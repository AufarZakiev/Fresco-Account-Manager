use std::collections::HashMap;
use std::sync::Mutex;

/// A mock BOINC project definition.
#[derive(Debug, Clone)]
pub struct MockProjectDef {
    pub url: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub general_area: &'static str,
    pub specific_area: &'static str,
    pub home_url: &'static str,
    pub web_rpc_url_base: Option<&'static str>,
    pub client_account_creation_disabled: bool,
    pub apps: &'static [MockApp],
    pub platforms: &'static [&'static str],
    pub project_specific_xml: &'static str,
}

#[derive(Debug, Clone)]
pub struct MockApp {
    pub name: &'static str,
    pub id: i64,
    pub user_friendly_name: &'static str,
}

// ── Project definitions ──

pub static MOCK_PROJECTS: &[MockProjectDef] = &[
    MockProjectDef {
        url: "https://einstein.phys.uwm.edu/",
        name: "Einstein@Home",
        description: "Search for spinning neutron stars using data from LIGO and radio telescopes",
        general_area: "Physical Science",
        specific_area: "Astrophysics",
        home_url: "https://einstein.phys.uwm.edu/",
        web_rpc_url_base: None,
        client_account_creation_disabled: false,
        apps: &[
            MockApp { name: "einstein_O4AS", id: 56, user_friendly_name: "O4 All-Sky" },
            MockApp { name: "einstein_O4MD", id: 59, user_friendly_name: "O4 Multi-Directed" },
            MockApp { name: "einsteinbinary_BRP7", id: 57, user_friendly_name: "Binary Radio Pulsar 7" },
            MockApp { name: "einsteinbinary_BRP4A", id: 60, user_friendly_name: "Binary Radio Pulsar 4A" },
            MockApp { name: "hsgamma_FGRP5", id: 46, user_friendly_name: "Gamma-ray pulsar FGRP5" },
            MockApp { name: "hsgamma_FGRPB1G", id: 40, user_friendly_name: "Gamma-ray pulsar FGRPB1G" },
            MockApp { name: "einsteinbinary_BRP4G", id: 25, user_friendly_name: "Binary Radio Pulsar 4 GPU" },
            MockApp { name: "einsteinbinary_BRP4", id: 19, user_friendly_name: "Binary Radio Pulsar 4" },
        ],
        platforms: &["windows_x86_64", "x86_64-pc-linux-gnu", "aarch64-unknown-linux-gnu", "arm-android-linux-gnu", "x86_64-apple-darwin"],
        project_specific_xml: "\
<project_specific>\n\
  <gpu_util_brp>1.0</gpu_util_brp>\n\
  <gpu_util_fgrp>1.0</gpu_util_fgrp>\n\
  <gpu_util_gw>1.0</gpu_util_gw>\n\
</project_specific>",
    },
    MockProjectDef {
        url: "https://www.primegrid.com/",
        name: "PrimeGrid",
        description: "Search for prime numbers of world-record size",
        general_area: "Mathematics",
        specific_area: "Number Theory",
        home_url: "https://www.primegrid.com/",
        web_rpc_url_base: None,
        client_account_creation_disabled: false,
        apps: &[
            MockApp { name: "llr321", id: 1, user_friendly_name: "321 Prime Search (LLR)" },
            MockApp { name: "llrCUL", id: 2, user_friendly_name: "Cullen Prime Search (LLR)" },
            MockApp { name: "llrWOO", id: 3, user_friendly_name: "Woodall Prime Search (LLR)" },
            MockApp { name: "llrPSP", id: 4, user_friendly_name: "Proth Prime Search (LLR)" },
            MockApp { name: "llrPPS", id: 5, user_friendly_name: "PPS (Mega) Prime Search (LLR)" },
            MockApp { name: "llrSOB", id: 6, user_friendly_name: "Seventeen or Bust (LLR)" },
            MockApp { name: "genefer16", id: 10, user_friendly_name: "Generalized Fermat 16" },
            MockApp { name: "genefer22", id: 11, user_friendly_name: "Generalized Fermat 22" },
            MockApp { name: "ap26", id: 20, user_friendly_name: "AP26 Search" },
        ],
        platforms: &["windows_x86_64", "x86_64-pc-linux-gnu", "x86_64-apple-darwin"],
        project_specific_xml: "\
<project_specific>\n\
  <321_pref>1</321_pref>\n\
  <cullen_pref>1</cullen_pref>\n\
  <woodall_pref>1</woodall_pref>\n\
  <psp_pref>1</psp_pref>\n\
  <pps_pref>1</pps_pref>\n\
  <sob_pref>1</sob_pref>\n\
  <gfn16_pref>1</gfn16_pref>\n\
  <gfn22_pref>1</gfn22_pref>\n\
  <ap26_pref>1</ap26_pref>\n\
  <send_if_no_work>1</send_if_no_work>\n\
</project_specific>",
    },
    MockProjectDef {
        url: "https://boinc.bakerlab.org/rosetta/",
        name: "Rosetta@home",
        description: "Determine the 3-dimensional shapes of proteins",
        general_area: "Life Sciences",
        specific_area: "Biology",
        home_url: "https://boinc.bakerlab.org/rosetta/",
        web_rpc_url_base: None,
        client_account_creation_disabled: false,
        apps: &[
            MockApp { name: "rosetta", id: 1, user_friendly_name: "Rosetta" },
            MockApp { name: "rosetta_beta", id: 2, user_friendly_name: "Rosetta Beta" },
            MockApp { name: "rosetta_mini", id: 3, user_friendly_name: "Rosetta Mini" },
            MockApp { name: "rosetta_python", id: 4, user_friendly_name: "Rosetta Python" },
        ],
        platforms: &["windows_x86_64", "x86_64-pc-linux-gnu", "x86_64-apple-darwin"],
        project_specific_xml: "<project_specific>\n</project_specific>",
    },
    MockProjectDef {
        url: "https://www.worldcommunitygrid.org/",
        name: "World Community Grid",
        description: "Multiple humanitarian research projects",
        general_area: "Multiple Areas",
        specific_area: "Multiple Applications",
        home_url: "https://www.worldcommunitygrid.org/",
        web_rpc_url_base: None,
        client_account_creation_disabled: true,
        apps: &[
            MockApp { name: "mam1", id: 1, user_friendly_name: "Mapping Cancer Markers" },
            MockApp { name: "opn1", id: 2, user_friendly_name: "OpenPandemics" },
            MockApp { name: "arp1", id: 3, user_friendly_name: "Africa Rainfall Project" },
            MockApp { name: "mcm1", id: 4, user_friendly_name: "Microbiome Immunity Project" },
        ],
        platforms: &["windows_x86_64", "x86_64-pc-linux-gnu", "x86_64-apple-darwin", "arm-android-linux-gnu"],
        project_specific_xml: "\
<project_specific>\n\
  <mam1_pref>1</mam1_pref>\n\
  <opn1_pref>1</opn1_pref>\n\
  <arp1_pref>1</arp1_pref>\n\
  <mcm1_pref>1</mcm1_pref>\n\
</project_specific>",
    },
    MockProjectDef {
        url: "https://lhcathome.cern.ch/lhcathome/",
        name: "LHC@home",
        description: "Simulate particle physics experiments at CERN",
        general_area: "Physical Science",
        specific_area: "High-Energy Physics",
        home_url: "https://lhcathome.cern.ch/lhcathome/",
        web_rpc_url_base: None,
        client_account_creation_disabled: false,
        apps: &[
            MockApp { name: "sixtrack", id: 1, user_friendly_name: "SixTrack" },
            MockApp { name: "atlas", id: 2, user_friendly_name: "ATLAS Simulation" },
            MockApp { name: "cms", id: 3, user_friendly_name: "CMS Simulation" },
            MockApp { name: "theory", id: 4, user_friendly_name: "Theory Simulation" },
        ],
        platforms: &["windows_x86_64", "x86_64-pc-linux-gnu", "x86_64-apple-darwin"],
        project_specific_xml: "\
<project_specific>\n\
  <sixtrack_pref>1</sixtrack_pref>\n\
  <atlas_pref>1</atlas_pref>\n\
  <cms_pref>1</cms_pref>\n\
  <theory_pref>1</theory_pref>\n\
</project_specific>",
    },
    MockProjectDef {
        url: "https://milkyway.cs.rpi.edu/milkyway/",
        name: "Milkyway@home",
        description: "Model the Milky Way galaxy using volunteer computing",
        general_area: "Physical Science",
        specific_area: "Astrophysics",
        home_url: "https://milkyway.cs.rpi.edu/milkyway/",
        web_rpc_url_base: None,
        client_account_creation_disabled: false,
        apps: &[
            MockApp { name: "milkyway_nbody", id: 1, user_friendly_name: "N-Body Simulation" },
            MockApp { name: "milkyway_separation", id: 2, user_friendly_name: "Separation" },
        ],
        platforms: &["windows_x86_64", "x86_64-pc-linux-gnu", "x86_64-apple-darwin"],
        project_specific_xml: "<project_specific>\n</project_specific>",
    },
    MockProjectDef {
        url: "https://gpugrid.net/gpugrid/",
        name: "GPUGRID",
        description: "Full-atom molecular simulations of proteins",
        general_area: "Life Sciences",
        specific_area: "Molecular Biology",
        home_url: "https://gpugrid.net/",
        web_rpc_url_base: None,
        client_account_creation_disabled: false,
        apps: &[
            MockApp { name: "ACEMD", id: 1, user_friendly_name: "ACEMD" },
        ],
        platforms: &["windows_x86_64", "x86_64-pc-linux-gnu"],
        project_specific_xml: "<project_specific>\n</project_specific>",
    },
    MockProjectDef {
        url: "https://climateprediction.net/",
        name: "climateprediction.net",
        description: "Study climate change through large-scale climate modelling",
        general_area: "Earth Sciences",
        specific_area: "Climate Study",
        home_url: "https://climateprediction.net/",
        // Intentionally different from master_url to test web_rpc_url_base edge case
        web_rpc_url_base: Some("https://cpdn-www.oerc.ox.ac.uk/cpdnboinc/"),
        client_account_creation_disabled: false,
        apps: &[
            MockApp { name: "cpdn_climate", id: 1, user_friendly_name: "Climate Model" },
            MockApp { name: "cpdn_openifs", id: 2, user_friendly_name: "OpenIFS Weather Model" },
        ],
        platforms: &["windows_x86_64", "x86_64-pc-linux-gnu"],
        project_specific_xml: "\
<project_specific>\n\
  <climate_model_pref>1</climate_model_pref>\n\
  <openifs_pref>1</openifs_pref>\n\
</project_specific>",
    },
    MockProjectDef {
        url: "https://numberfields.asu.edu/NumberFields/",
        name: "NumberFields@home",
        description: "Research in number theory",
        general_area: "Mathematics",
        specific_area: "Number Theory",
        home_url: "https://numberfields.asu.edu/NumberFields/",
        web_rpc_url_base: None,
        client_account_creation_disabled: false,
        apps: &[
            MockApp { name: "numberfields", id: 1, user_friendly_name: "Number Fields" },
        ],
        platforms: &["windows_x86_64", "x86_64-pc-linux-gnu", "x86_64-apple-darwin"],
        project_specific_xml: "<project_specific>\n</project_specific>",
    },
    MockProjectDef {
        url: "https://www.asteroids@home.net/boinc/",
        name: "Asteroids@home",
        description: "Derive shapes and spin properties of asteroids",
        general_area: "Physical Science",
        specific_area: "Astronomy",
        home_url: "https://www.asteroids@home.net/",
        web_rpc_url_base: None,
        client_account_creation_disabled: false,
        apps: &[
            MockApp { name: "period_search", id: 1, user_friendly_name: "Period Search" },
        ],
        platforms: &["windows_x86_64", "x86_64-pc-linux-gnu", "x86_64-apple-darwin"],
        project_specific_xml: "<project_specific>\n</project_specific>",
    },
    MockProjectDef {
        url: "https://universeathome.pl/universe/",
        name: "Universe@Home",
        description: "Astrophysical simulations of stellar and galactic evolution",
        general_area: "Physical Science",
        specific_area: "Astrophysics",
        home_url: "https://universeathome.pl/universe/",
        web_rpc_url_base: None,
        client_account_creation_disabled: false,
        apps: &[
            MockApp { name: "universe", id: 1, user_friendly_name: "Universe Simulation" },
        ],
        platforms: &["windows_x86_64", "x86_64-pc-linux-gnu"],
        project_specific_xml: "<project_specific>\n</project_specific>",
    },
];

/// In-memory store for mock project preferences (keyed by (project_url, account_key)).
pub struct MockProjectStore {
    prefs: Mutex<HashMap<(String, String), String>>,
}

impl MockProjectStore {
    pub fn new() -> Self {
        Self {
            prefs: Mutex::new(HashMap::new()),
        }
    }

    /// Find a mock project definition by URL prefix match.
    pub fn find_project(project_url: &str) -> Option<&'static MockProjectDef> {
        let normalized = normalize_url(project_url);
        MOCK_PROJECTS.iter().find(|p| normalize_url(p.url) == normalized)
    }

    /// Generate a deterministic mock authenticator.
    fn mock_authenticator(project_url: &str, email: &str) -> String {
        use md5::{Digest, Md5};
        let mut hasher = Md5::new();
        hasher.update(project_url.as_bytes());
        hasher.update(email.as_bytes());
        let result = hasher.finalize();
        format!("mock-auth-{}", hex::encode(result))
    }

    /// Mock `create_account.php` response.
    pub fn create_account(
        &self,
        project_url: &str,
        email: &str,
        _passwd_hash: &str,
        _user_name: &str,
    ) -> Result<String, MockError> {
        let def = Self::find_project(project_url)
            .ok_or(MockError::UnknownProject)?;

        if def.client_account_creation_disabled {
            return Err(MockError::ProjectError {
                code: -208,
                message: "Account creation disabled".to_string(),
            });
        }

        Ok(Self::mock_authenticator(project_url, email))
    }

    /// Mock `lookup_account.php` response.
    pub fn lookup_account(
        &self,
        project_url: &str,
        email: &str,
        _passwd_hash: &str,
    ) -> Result<String, MockError> {
        Self::find_project(project_url)
            .ok_or(MockError::UnknownProject)?;

        Ok(Self::mock_authenticator(project_url, email))
    }

    /// Mock `am_get_info.php` response.
    pub fn get_info(
        &self,
        project_url: &str,
        account_key: &str,
    ) -> Result<MockAccountInfo, MockError> {
        let def = Self::find_project(project_url)
            .ok_or(MockError::UnknownProject)?;

        let project_prefs = {
            let store = self.prefs.lock().unwrap();
            let key = (project_url.to_string(), account_key.to_string());
            store.get(&key).cloned()
                .unwrap_or_else(|| build_default_project_prefs(def))
        };

        Ok(MockAccountInfo {
            id: 12345,
            name: "Mock User".to_string(),
            country: "International".to_string(),
            weak_auth: format!("mock-weak-{}", &account_key[..8.min(account_key.len())]),
            teamid: 0,
            project_prefs,
        })
    }

    /// Mock `am_set_info.php` — stores project prefs in memory.
    pub fn set_info(
        &self,
        project_url: &str,
        account_key: &str,
        project_prefs: Option<&str>,
    ) -> Result<(), MockError> {
        Self::find_project(project_url)
            .ok_or(MockError::UnknownProject)?;

        if let Some(prefs) = project_prefs {
            let mut store = self.prefs.lock().unwrap();
            store.insert(
                (project_url.to_string(), account_key.to_string()),
                prefs.to_string(),
            );
        }

        Ok(())
    }

    /// Mock `get_project_config.php` response.
    pub fn get_project_config(project_url: &str) -> Result<MockProjectConfig, MockError> {
        let def = Self::find_project(project_url)
            .ok_or(MockError::UnknownProject)?;

        let web_rpc_url_base = def.web_rpc_url_base
            .map(|s| s.to_string())
            .unwrap_or_else(|| project_url.to_string());

        Ok(MockProjectConfig {
            name: def.name.to_string(),
            web_rpc_url_base,
            client_account_creation_disabled: def.client_account_creation_disabled,
            apps: def.apps.iter().map(|a| MockConfigApp {
                name: a.name.to_string(),
                id: a.id,
                user_friendly_name: a.user_friendly_name.to_string(),
            }).collect(),
            platforms: def.platforms.iter().map(|p| p.to_string()).collect(),
        })
    }
}

impl Default for MockProjectStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Build default project_prefs XML with all apps selected.
fn build_default_project_prefs(def: &MockProjectDef) -> String {
    let mut xml = String::from("<project_preferences>\n");
    if !def.apps.is_empty() {
        xml.push_str("  <apps_selected>\n");
        for app in def.apps {
            xml.push_str(&format!("    <app_id>{}</app_id>\n", app.id));
        }
        xml.push_str("  </apps_selected>\n");
    }
    xml.push_str("  <allow_beta_work>0</allow_beta_work>\n");
    xml.push_str("  <allow_non_preferred_apps>1</allow_non_preferred_apps>\n");
    xml.push_str(def.project_specific_xml);
    xml.push('\n');
    xml.push_str("</project_preferences>");
    xml
}

fn normalize_url(url: &str) -> String {
    let url = url.trim_end_matches('/');
    url.to_lowercase()
}

// ── Return types ──

#[derive(Debug, Clone)]
pub struct MockAccountInfo {
    pub id: i64,
    pub name: String,
    pub country: String,
    pub weak_auth: String,
    pub teamid: i64,
    pub project_prefs: String,
}

#[derive(Debug, Clone)]
pub struct MockProjectConfig {
    pub name: String,
    pub web_rpc_url_base: String,
    pub client_account_creation_disabled: bool,
    pub apps: Vec<MockConfigApp>,
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct MockConfigApp {
    pub name: String,
    pub id: i64,
    pub user_friendly_name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum MockError {
    #[error("unknown mock project")]
    UnknownProject,
    #[error("project error {code}: {message}")]
    ProjectError { code: i32, message: String },
}

// Build XML responses for the mock endpoints (used in project_rpc.rs mock mode)

impl MockProjectStore {
    pub fn build_create_account_xml(
        &self,
        project_url: &str,
        email: &str,
        passwd_hash: &str,
        user_name: &str,
    ) -> String {
        match self.create_account(project_url, email, passwd_hash, user_name) {
            Ok(auth) => format!(
                "<account_out>\n  <authenticator>{auth}</authenticator>\n</account_out>"
            ),
            Err(MockError::ProjectError { code, message }) => format!(
                "<error>\n  <error_num>{code}</error_num>\n  <message>{message}</message>\n</error>"
            ),
            Err(_) => "<error>\n  <error_num>-1</error_num>\n  <message>Unknown error</message>\n</error>".to_string(),
        }
    }

    pub fn build_lookup_account_xml(
        &self,
        project_url: &str,
        email: &str,
        passwd_hash: &str,
    ) -> String {
        match self.lookup_account(project_url, email, passwd_hash) {
            Ok(auth) => format!(
                "<account_out>\n  <authenticator>{auth}</authenticator>\n</account_out>"
            ),
            Err(MockError::ProjectError { code, message }) => format!(
                "<error>\n  <error_num>{code}</error_num>\n  <message>{message}</message>\n</error>"
            ),
            Err(_) => "<error>\n  <error_num>-136</error_num>\n  <message>Not found</message>\n</error>".to_string(),
        }
    }

    pub fn build_get_info_xml(
        &self,
        project_url: &str,
        account_key: &str,
    ) -> String {
        match self.get_info(project_url, account_key) {
            Ok(info) => format!(
                "<am_get_info_reply>\n  <success/>\n  <id>{}</id>\n  <name>{}</name>\n  \
                 <country>{}</country>\n  <weak_auth>{}</weak_auth>\n  <teamid>{}</teamid>\n  \
                 {}\n</am_get_info_reply>",
                info.id, info.name, info.country, info.weak_auth, info.teamid, info.project_prefs,
            ),
            Err(MockError::ProjectError { code, .. }) => format!(
                "<am_get_info_reply>\n  <error_num>{code}</error_num>\n</am_get_info_reply>"
            ),
            Err(_) => "<am_get_info_reply>\n  <error_num>-1</error_num>\n</am_get_info_reply>".to_string(),
        }
    }

    pub fn build_set_info_xml(
        &self,
        project_url: &str,
        account_key: &str,
        project_prefs: Option<&str>,
    ) -> String {
        match self.set_info(project_url, account_key, project_prefs) {
            Ok(()) => "<am_set_info_reply>\n  <success/>\n</am_set_info_reply>".to_string(),
            Err(MockError::ProjectError { code, .. }) => format!(
                "<am_set_info_reply>\n  <error_num>{code}</error_num>\n</am_set_info_reply>"
            ),
            Err(_) => "<am_set_info_reply>\n  <error_num>-1</error_num>\n</am_set_info_reply>".to_string(),
        }
    }

    pub fn build_project_config_xml(project_url: &str) -> String {
        match Self::get_project_config(project_url) {
            Ok(config) => {
                let mut xml = String::from("<project_config>\n");
                xml.push_str(&format!("  <name>{}</name>\n", config.name));
                xml.push_str(&format!("  <web_rpc_url_base>{}</web_rpc_url_base>\n", config.web_rpc_url_base));
                if config.client_account_creation_disabled {
                    xml.push_str("  <client_account_creation_disabled/>\n");
                }
                for app in &config.apps {
                    xml.push_str("  <app>\n");
                    xml.push_str(&format!("    <name>{}</name>\n", app.name));
                    xml.push_str(&format!("    <id>{}</id>\n", app.id));
                    xml.push_str(&format!("    <user_friendly_name>{}</user_friendly_name>\n", app.user_friendly_name));
                    xml.push_str("  </app>\n");
                }
                for platform in &config.platforms {
                    xml.push_str(&format!("  <platform>{platform}</platform>\n"));
                }
                xml.push_str("</project_config>");
                xml
            }
            Err(_) => "<error>\n  <error_num>-1</error_num>\n</error>".to_string(),
        }
    }
}
