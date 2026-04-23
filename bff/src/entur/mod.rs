pub mod geocoder;
pub mod journey;
pub mod service_journey;

use reqwest::Client;

const ET_CLIENT_NAME: &str = "kyss-app";

#[derive(Clone)]
pub struct EnturClient {
    http: Client,
}

impl EnturClient {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }
}
