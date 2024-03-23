use std::path::PathBuf;

use crate::APP_NAME;
use clap::builder::PossibleValue;
use config::Config;
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CfgOutputFormat {
    YAML,
}

impl Default for &CfgOutputFormat {
    fn default() -> Self {
        &CfgOutputFormat::YAML
    }
}

impl clap::ValueEnum for CfgOutputFormat {
    fn value_variants<'a>() -> &'a [Self] {
        &[CfgOutputFormat::YAML]
    }

    fn to_possible_value(&self) -> Option<clap::builder::PossibleValue> {
        Some(match self {
            CfgOutputFormat::YAML => PossibleValue::new("yaml").help("YAML"),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Cfg {
    pub verbose: String,
    pub address: String,
    pub port: u16,
    pub template_glob: String,
    pub storage_path: Option<String>,
}

impl Default for Cfg {
    fn default() -> Self {
        Cfg {
            verbose: "info".to_string(),
            address: "0.0.0.0".to_string(),
            port: 8080,
            template_glob: default_template_glob(),
            #[cfg(feature = "csv")]
            storage_path: None,
            #[cfg(feature = "sqlite")]
            storage_path: None,
        }
    }
}

impl std::fmt::Display for Cfg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<Config> for Cfg {
    fn from(value: Config) -> Self {
        let mut cfg = Cfg::default();
        if let Ok(o) = value.get_string("verbose") {
            cfg.verbose = o;
        }
        if let Ok(o) = value.get_string("address") {
            cfg.address = o;
        }
        if let Ok(o) = value.get_int("port") {
            cfg.port = o as u16;
        }
        if let Ok(o) = value.get_string("template_glob") {
            cfg.template_glob = o;
        }
        // FUTURE add more parsing for new fields added to Cfg struct
        cfg
    }
}

#[allow(dead_code)]
pub fn write_cfg(out: &mut dyn Write, settings: &Cfg, fmt: &CfgOutputFormat) {
    match fmt {
        CfgOutputFormat::YAML => writeln!(
            out,
            "{}",
            serde_yaml::to_string(&settings).expect("Failed to serialize settings to YAML")
        )
        .expect("Failed to write config to stdout"),
    }
}

/// Returns the default configuration file path for the FIXME.
///
/// The default configuration file path is determined by appending
/// `".config/FIXME/default.yaml"` to the account's home directory.
///
/// # Examples
///
/// ```
/// use crate::default_config_path;
///
/// let path = default_config_path();
/// println!("Default configuration file path: {:?}", path);
/// ```
///
/// # Errors
///
/// This function will panic if it fails to retrieve the account's home directory
/// using the `UserDirs` struct from the `directories` crate.
///
/// # Returns
///
/// The function returns a `PathBuf` representing the default configuration file path.
///
/// # Safety
///
/// This function assumes that the `UserDirs` struct from the `directories` crate
/// is capable of correctly retrieving the account's home directory.
///
/// # Dependencies
///
/// This function depends on the following crates:
///
/// - `std::path::PathBuf` - For manipulating file paths.
/// - `directories` - For retrieving the account's home directory.
///
/// # Panics
///
/// This function will panic if it fails to retrieve the account's home directory.
///
/// # Notes
///
/// It is recommended to handle the potential errors when using this function.
///
#[allow(dead_code)]
pub fn default_config_path() -> PathBuf {
    let user_dirs = UserDirs::new().unwrap();
    let mut path = PathBuf::from(user_dirs.home_dir());
    path.push(format!(".config/{APP_NAME}/default.yaml"));
    path
}

#[allow(dead_code)]
pub fn default_template_glob() -> String {
    concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*").to_string()
}

#[cfg(test)]
mod tests {
    use unindent::unindent;

    use super::*;

    #[test]
    fn writing_default_cfg_as_yaml() {
        let expected = format!(
            r#"
        verbose: info
        address: 0.0.0.0
        port: 8080
        template_glob: {}
        storage_path: null

        "#,
            default_template_glob()
        );
        let mut actual = Vec::new();
        let settings = Cfg::default();
        write_cfg(&mut actual, &settings, &CfgOutputFormat::YAML);
        assert_eq!(unindent(&expected), String::from_utf8_lossy(&actual));
    }
}
