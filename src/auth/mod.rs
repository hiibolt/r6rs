use base64::prelude::*;
use reqwest::header::HeaderMap;
use crate::Value;
use std::collections::HashMap;
use std::error::Error;
use reqwest::StatusCode;

#[derive(Debug)]
pub struct UbisoftAPI {
    email: String,
    password: String,
    token: String,

    space_ids: HashMap<String, String>,
    headers: HeaderMap
}
impl UbisoftAPI {
    fn get_basic_token ( email: String, password: String ) -> String {
        BASE64_STANDARD
            .encode(format!("{}:{}", email, password))
    }

    pub fn new ( email: String, password: String ) -> Self {
        let space_ids = HashMap::from([
            ("uplay".to_string(), "0d2ae42d-4c27-4cb7-af6c-2099062302bb".to_string()),
            ("psn".to_string(), "0d2ae42d-4c27-4cb7-af6c-2099062302bb".to_string()),
            ("xbl".to_string(), "0d2ae42d-4c27-4cb7-af6c-2099062302bb".to_string())
        ]);
        let token = Self::get_basic_token( email.clone(), password.clone() );

        Self {
            email,
            password,
            token,

            space_ids,
            headers: HeaderMap::new()
        }
    }
    pub async fn login ( &mut self ) -> Result<(), Box<dyn Error>> {
        let client = reqwest::Client::new();

        self.headers.insert("Authorization", format!("Basic {}", self.token).parse()?);
        self.headers.insert("User-Agent", "UbiServices_SDK_2020.Release.58_PC64_ansi_static".parse()?);
        self.headers.insert("Content-Type", "application/json; charset=UTF-8".parse()?);
        self.headers.insert("Ubi-AppId", "e3d5ea9e-50bd-43b7-88bf-39794f4e3d40".parse()?);
        self.headers.insert("Ubi-LocaleCode", "en-us".parse()?);

        let request = client.post("https://public-ubiservices.ubi.com/v2/profiles/sessions")
            .headers(self.headers.clone())
            .body("{\"rememberMe\": true}");
        
        let response = request
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                let response_json: Value = serde_json::from_str(&response.text().await?)?;

                *(self.headers.get_mut("Authorization")
                    .ok_or("Unreachable")?) = format!("Ubi_v1 t={}", response_json["ticket"].as_str().ok_or("Ticket missing from Ubi response!")?).parse()?;
        
                println!("Successfully authenticated!");
            },
            _ => {
                panic!("Failed to authenticate with given login! Verify your information is correct.");
            }
        }

        Ok(())
    }

    pub async fn basic_request ( &mut self, url: String ) -> Result<Value, Box<dyn Error>> {
        let client = reqwest::Client::new();

        let request = client.get(&url)
            .headers(self.headers.clone());
        
        let response = request
            .send()
            .await?;

        Ok(serde_json::from_str(&response.text().await?)?)
    }
}