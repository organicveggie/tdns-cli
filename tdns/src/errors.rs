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

    fn make_http_error(&self, error: reqwest::Error) -> TdnsError {
        TdnsError::HttpRequestError {
            command: self.get_command_name().to_string(),
            host: self.get_host().to_string(),
            source: error,
        }
    }

    fn make_json_error(&self, error: serde_json::Error) -> TdnsError {
        TdnsError::JsonError {
            command: self.get_command_name().to_string(),
            source: error,
        }
    }

    fn make_config_error(&self, error: crate::config::ConfigFileError) -> TdnsError {
        TdnsError::ConfigFileError {
            command: self.get_command_name().to_string(),
            source: error,
        }
    }
}
