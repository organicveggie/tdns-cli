#[derive(Debug, thiserror::Error)]
pub enum TdnsError {
    #[error("{command} HTTP request to {host} failed")]
    HttpRequestError {
        command: String,
        host: String,
        source: reqwest::Error,
    },

    #[error("{command} JSON deserialization failed")]
    JsonError {
        command: String,
        source: serde_json::Error,
    },

    #[error("{command} config file error")]
    ConfigFileError {
        command: String,
        source: crate::config::ConfigFileError,
    },

    #[error("{command} invalid domain name: {domain} - {message}")]
    InvalidDomainName {
        command: String,
        domain: String,
        message: String,
    },

    #[error("{command} output error")]
    OutputError {
        command: String,
        source: std::io::Error,
    },
}

pub trait TdnsErrorGenerator {
    fn get_command_name(&self) -> &str;
    fn get_host(&self) -> &str;

    fn make_http_err(&self, error: reqwest::Error) -> Result<(), TdnsError> {
        Err(self.make_http_error(error))
    }

    fn make_json_err(&self, error: serde_json::Error) -> Result<(), TdnsError> {
        Err(self.make_json_error(error))
    }

    fn make_config_err(&self, error: crate::config::ConfigFileError) -> Result<(), TdnsError> {
        Err(self.make_config_error(error))
    }

    fn make_output_err(&self, error: std::io::Error) -> Result<(), TdnsError> {
        Err(self.make_output_error(error))
    }

    fn make_http_error(&self, error: reqwest::Error) -> TdnsError {
        make_http_error(self.get_command_name(), self.get_host(), error)
    }

    fn make_json_error(&self, error: serde_json::Error) -> TdnsError {
        make_json_error(self.get_command_name(), error)
    }

    fn make_config_error(&self, error: crate::config::ConfigFileError) -> TdnsError {
        make_config_error(self.get_command_name(), error)
    }

    fn make_output_error(&self, error: std::io::Error) -> TdnsError {
        make_output_error(self.get_command_name(), error)
    }
}

pub fn make_http_error(command: &str, host: &str, error: reqwest::Error) -> TdnsError {
    TdnsError::HttpRequestError {
        command: command.to_string(),
        host: host.to_string(),
        source: error,
    }
}

pub fn make_json_error(command: &str, error: serde_json::Error) -> TdnsError {
    TdnsError::JsonError {
        command: command.to_string(),
        source: error,
    }
}

pub fn make_config_error(command: &str, error: crate::config::ConfigFileError) -> TdnsError {
    TdnsError::ConfigFileError {
        command: command.to_string(),
        source: error,
    }
}

pub fn make_output_error(command: &str, error: std::io::Error) -> TdnsError {
    TdnsError::OutputError {
        command: command.to_string(),
        source: error,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_config_error_populates_fields() {
        let command_name = "TestCommand";
        let config_error = crate::config::ConfigFileError::FileReadError {
            file_name: "config.toml".to_string(),
            error: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
        };
        let tdns_error = make_config_error(command_name, config_error);

        match tdns_error {
            TdnsError::ConfigFileError { command, source: _ } => {
                assert_eq!(command, command_name);
            }
            _ => panic!("Expected TdnsError::ConfigFileError variant"),
        }
    }

    #[test]
    fn make_json_error_populates_fields() {
        let command_name = "TestCommand";
        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let tdns_error = make_json_error(command_name, json_error);

        match tdns_error {
            TdnsError::JsonError { command, source: _ } => {
                assert_eq!(command, command_name);
            }
            _ => panic!("Expected TdnsError::JsonError variant"),
        }
    }

    fn trait_test_error_generator<T: TdnsErrorGenerator>(generator: T) {
        let config_error = crate::config::ConfigFileError::FileReadError {
            file_name: "config.toml".to_string(),
            error: std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"),
        };

        let tdns_error = generator.make_config_err(config_error);
        assert!(tdns_error.is_err());
        match tdns_error.unwrap_err() {
            TdnsError::ConfigFileError { command, source: _ } => {
                assert_eq!(command, generator.get_command_name());
            }
            _ => panic!("Expected TdnsError::ConfigFileError variant"),
        }

        let json_error = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let tdns_error = generator.make_json_err(json_error);
        assert!(tdns_error.is_err());
        match tdns_error.unwrap_err() {
            TdnsError::JsonError { command, source: _ } => {
                assert_eq!(command, generator.get_command_name());
            }
            _ => panic!("Expected TdnsError::JsonError variant"),
        }
    }

    mod trait_tests {
        use super::*;

        struct TestGenerator;

        impl TdnsErrorGenerator for TestGenerator {
            fn get_command_name(&self) -> &str {
                "TestCommand"
            }

            fn get_host(&self) -> &str {
                "testhost"
            }
        }

        #[test]
        fn make_errors_return_errors() {
            let generator = TestGenerator;
            trait_test_error_generator(generator);
        }
    }
}
