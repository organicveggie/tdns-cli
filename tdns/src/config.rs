use std::fs::{self, File, create_dir_all};
use std::io::Write;
use std::path::Path;
use std::rc::Rc;

use mockall::automock;
use serde::{Deserialize, Serialize};

use crate::client;

pub struct ApplicationConfig {
    pub config_manager: Box<dyn ConfigManager>,
    pub tdns_client: Rc<dyn client::TdnsClient>,
}

#[automock]
pub trait ConfigManager {
    fn create_config_file(&self, file_name: &str, force: &bool) -> Result<(), ConfigFileError>;
    fn read_config_file(&self, file_name: &str) -> Result<Config, ConfigFileError>;
}

pub struct ConfigFileManager;

impl ConfigManager for ConfigFileManager {
    fn create_config_file(&self, file_name: &str, force: &bool) -> Result<(), ConfigFileError> {
        let file_path = Path::new(&file_name);
        match file_path.try_exists() {
            Ok(exists) => {
                if exists {
                    if !force {
                        return Err(ConfigFileError::FileExistsError {
                            file_name: file_name.to_string(),
                        });
                    }
                    println!("Overwriting existing config file: {:?}", file_path);
                } else {
                    println!("File does not exist: {:?}", file_path);
                }
            }
            Err(error) => {
                return Err(ConfigFileError::FileStatusError {
                    file_name: file_name.to_string(),
                    source: error,
                });
            }
        }

        // Validate parent directory
        let dir_path = match file_path.parent() {
            Some(dir_path) => dir_path,
            None => {
                return Err(ConfigFileError::InvalidDirectoryError {
                    file_name: file_name.to_string(),
                });
            }
        };

        let dir_name = match dir_path.to_str() {
            Some(s) => s,
            None => "",
        };

        // Check if parent directory exists
        let exists = match dir_path.try_exists() {
            Ok(exists) => exists,
            Err(error) => {
                return Err(ConfigFileError::FileStatusError {
                    file_name: dir_name.to_string(),
                    source: error,
                });
            }
        };

        // Create directory if it doesn't exist
        if !exists {
            match create_dir_all(dir_path) {
                Ok(()) => println!("Created directory: {:?}", dir_path),
                Err(error) => {
                    return Err(ConfigFileError::CreateDirectoryError {
                        directory_name: dir_name.to_string(),
                        source: error,
                    });
                }
            }
        }

        // Double check the directory is a directory.
        if !dir_path.is_dir() {
            return Err(ConfigFileError::InvalidConfigDirectory {
                directory_name: dir_name.to_string(),
            });
        }

        let mut config_file = File::create(file_path).unwrap();
        let config = Config {
            token: "".to_owned(),
            host: "".to_owned(),
        };
        let js = match serde_json::to_string_pretty(&config) {
            Ok(s) => s,
            Err(error) => return Err(ConfigFileError::JsonSerializeError { source: error }),
        };
        match config_file.write_all(js.as_bytes()) {
            Err(error) => {
                return Err(ConfigFileError::JsonWriteError {
                    file_name: file_name.to_string(),
                    error,
                });
            }
            _ => {}
        }

        Ok(())
    }

    fn read_config_file(&self, file_name: &str) -> Result<Config, ConfigFileError> {
        let contents = match fs::read_to_string(file_name) {
            Ok(s) => s,
            Err(error) => {
                return Err(ConfigFileError::FileReadError {
                    file_name: file_name.to_string(),
                    error,
                });
            }
        };

        let config: Config = match serde_json::from_str(&contents) {
            Ok(c) => c,
            Err(error) => {
                return Err(ConfigFileError::JsonDeserializeError {
                    file_name: file_name.to_string(),
                    error,
                });
            }
        };
        Ok(Config::normalize(&config))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    token: String,
    host: String,
}

impl Config {
    pub fn new(host: &str, token: &str) -> Config {
        let host_name = match host.strip_suffix("/") {
            Some(h) => h,
            None => host,
        };
        Config {
            token: token.to_string(),
            host: host_name.to_string(),
        }
    }

    pub fn normalize(config: &Config) -> Config {
        Config::new(config.host.as_str(), config.token.as_str())
    }

    pub fn get_host(&self) -> &str {
        self.host.as_str()
    }
    pub fn get_token(&self) -> &str {
        self.token.as_str()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigFileError {
    #[error("config file exists {file_name}, but --force not specified!")]
    FileExistsError { file_name: String },

    #[error("failed to check existence of config file {file_name}")]
    FileStatusError {
        file_name: String,
        source: std::io::Error,
    },

    #[error("unable to extract parent directory from config file name: {file_name}")]
    InvalidDirectoryError { file_name: String },

    #[error("error creating config directory {directory_name}")]
    CreateDirectoryError {
        directory_name: String,
        source: std::io::Error,
    },

    #[error("config directory exists but is not a directory: {directory_name}")]
    InvalidConfigDirectory { directory_name: String },

    #[error("error deserializing config file from JSON file: {file_name}")]
    JsonDeserializeError {
        file_name: String,
        #[source]
        error: serde_json::Error,
    },

    #[error("error serializing config file into JSON")]
    JsonSerializeError { source: serde_json::Error },

    #[error("error writing JSON into config file: {file_name}")]
    JsonWriteError {
        file_name: String,
        #[source]
        error: std::io::Error,
    },

    #[error("error reading config file: {file_name}")]
    FileReadError {
        file_name: String,
        #[source]
        error: std::io::Error,
    },
}

pub fn create_config_file(file_name: &str, force: &bool) -> Result<(), ConfigFileError> {
    let file_path = Path::new(&file_name);
    match file_path.try_exists() {
        Ok(exists) => {
            if exists {
                if !force {
                    return Err(ConfigFileError::FileExistsError {
                        file_name: file_name.to_string(),
                    });
                }
                println!("Overwriting existing config file: {:?}", file_path);
            } else {
                println!("File does not exist: {:?}", file_path);
            }
        }
        Err(error) => {
            return Err(ConfigFileError::FileStatusError {
                file_name: file_name.to_string(),
                source: error,
            });
        }
    }

    // Validate parent directory
    let dir_path = match file_path.parent() {
        Some(dir_path) => dir_path,
        None => {
            return Err(ConfigFileError::InvalidDirectoryError {
                file_name: file_name.to_string(),
            });
        }
    };

    let dir_name = match dir_path.to_str() {
        Some(s) => s,
        None => "",
    };

    // Check if parent directory exists
    let exists = match dir_path.try_exists() {
        Ok(exists) => exists,
        Err(error) => {
            return Err(ConfigFileError::FileStatusError {
                file_name: dir_name.to_string(),
                source: error,
            });
        }
    };

    // Create directory if it doesn't exist
    if !exists {
        match create_dir_all(dir_path) {
            Ok(()) => println!("Created directory: {:?}", dir_path),
            Err(error) => {
                return Err(ConfigFileError::CreateDirectoryError {
                    directory_name: dir_name.to_string(),
                    source: error,
                });
            }
        }
    }

    // Double check the directory is a directory.
    if !dir_path.is_dir() {
        return Err(ConfigFileError::InvalidConfigDirectory {
            directory_name: dir_name.to_string(),
        });
    }

    let mut config_file = File::create(file_path).unwrap();
    let config = Config {
        token: "".to_owned(),
        host: "".to_owned(),
    };
    let js = match serde_json::to_string_pretty(&config) {
        Ok(s) => s,
        Err(error) => return Err(ConfigFileError::JsonSerializeError { source: error }),
    };
    match config_file.write_all(js.as_bytes()) {
        Err(error) => {
            return Err(ConfigFileError::JsonWriteError {
                file_name: file_name.to_string(),
                error,
            });
        }
        _ => {}
    }

    Ok(())
}

pub fn read_config_file(file_name: &str) -> Result<Config, ConfigFileError> {
    let contents = match fs::read_to_string(file_name) {
        Ok(s) => s,
        Err(error) => {
            return Err(ConfigFileError::FileReadError {
                file_name: file_name.to_string(),
                error,
            });
        }
    };

    let config: Config = match serde_json::from_str(&contents) {
        Ok(c) => c,
        Err(error) => {
            return Err(ConfigFileError::JsonDeserializeError {
                file_name: file_name.to_string(),
                error,
            });
        }
    };
    Ok(Config::normalize(&config))
}
