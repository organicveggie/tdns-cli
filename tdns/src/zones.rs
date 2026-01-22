use reqwest::Client;

use crate::config;

pub struct ListCmd {
    config: config::Config,
}

impl ListCmd {
    pub fn create(config_file: &str) -> Result<ListCmd, config::ConfigFileError> {
        let cfg = config::read_config_file(config_file)?;
        Ok(ListCmd { config: cfg })
    }

    pub async fn execute(&self) -> Result<(), reqwest::Error> {
        let client = Client::builder()
            .danger_accept_invalid_certs(true)
            .build()?;

        let host = self.config.get_host();
        let base_url = format!("{host}/api/zones/list?token={}", self.config.get_token());

        let body = client.get(base_url).send().await?.text().await?;
        println!("{}", body);

        Ok(())
    }
}
