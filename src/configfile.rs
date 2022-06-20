//! Primitives for operating on application configuration file
use serde::{Deserialize, Serialize};
use std::fs::{create_dir, File};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{error::Error, fmt::Debug};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Serialize, Deserialize, Debug)]
pub struct Configuration {
    /// Primary authentication token used to connect to OneDrive
    /// If this token expires we need to use the refresh_token
    /// to renew it
    pub auth_token: String,
    /// Secondary authentication token used to renew the lifetime
    /// of the primary authentication token
    pub refresh_token: String,
}

impl Configuration {
    /// Constructs an instance of the Configuraetion class fro YAML formatted
    /// data stored on disk
    ///
    /// # Arguments
    ///
    /// * `src_file` - Path to the YAML configuration file to parse
    pub fn from_file(src_file: &PathBuf) -> MyResult<Configuration> {
        let mut file = File::open(src_file)?;
        let mut s = String::new();
        file.read_to_string(&mut s)?;
        Ok(serde_yaml::from_str(&s)?)
    }

    /// Serializes an instance of the Configuration class to a YAML formatted
    /// source file
    ///
    /// # Arguments
    ///
    /// * `dest_file` - Path to the output file to serialize the config options
    ///                 to. Will conform to YAML encoding standards
    pub fn save(&self, dest_file: &PathBuf) -> MyResult<()> {
        let s = serde_yaml::to_string(&self)?;
        if !dest_file.parent().unwrap().is_dir() {
            create_dir(dest_file.parent().unwrap())?;
        }
        let mut file = File::create(dest_file)?;
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o600);
        file.set_permissions(perms)?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }
}

//-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
//                              UNIT TESTS
//-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=-=
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn save_config_file() {
        let temp_file = tempdir().unwrap().into_path().join("test.yml");
        let expected_auth_token = "abcd".to_string();
        let expected_refresh_token = "1234".to_string();
        let config = Configuration {
            auth_token: expected_auth_token.clone(),
            refresh_token: expected_refresh_token.clone(),
        };
        config.save(&temp_file).unwrap();

        let mut actual_data = String::new();
        File::open(temp_file)
            .unwrap()
            .read_to_string(&mut actual_data)
            .unwrap();

        assert!(actual_data.contains(&expected_auth_token));
        assert!(actual_data.contains(&expected_refresh_token));
    }

    #[test]
    fn load_config_file() {
        let test_file = [
            env!("CARGO_MANIFEST_DIR"),
            "src",
            "test_data",
            "config_files",
            "complete_config_file.yml",
        ]
        .iter()
        .collect();
        let config = Configuration::from_file(&test_file).unwrap();
        assert_eq!(config.auth_token, "abcdABCD");
        assert_eq!(config.refresh_token, "1234");
    }

    #[test]
    fn load_incomplete_config_file() {
        let test_file = [
            env!("CARGO_MANIFEST_DIR"),
            "src",
            "test_data",
            "config_files",
            "incomplete_config_file.yml",
        ]
        .iter()
        .collect();
        let result = Configuration::from_file(&test_file);

        assert!(result.is_err());
    }
}
