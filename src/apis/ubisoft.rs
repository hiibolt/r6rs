use base64::prelude::*;
use reqwest::header::HeaderMap;
use crate::Value;
use std::error::Error;
use reqwest::StatusCode;
use tokio::time::{ sleep, Duration };
use crate::{ Arc, Mutex };

#[derive(Debug)]
pub struct Ubisoft {
    token: String,
    headers: HeaderMap
}
impl Ubisoft {
    fn get_basic_token ( email: String, password: String ) -> String {
        BASE64_STANDARD
            .encode(format!("{}:{}", email, password))
    }

    pub fn new ( email: String, password: String ) -> Self {
        let token = Self::get_basic_token( email.clone(), password.clone() );

        Self {
            token,
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

                self.headers.insert("Authorization", format!("Ubi_v1 t={}", response_json["ticket"].as_str().ok_or("Ticket missing from Ubi response!")?).parse()?);
                self.headers.insert("Ubi-SessionId", response_json["sessionId"].as_str().ok_or("Ticket missing from Ubi response!")?.parse()?);
    
                println!("Successfully authenticated!");
            },
            _ => {
                panic!("Failed to authenticate with given login! Verify your information is correct.");
            }
        }

        Ok(())
    }
    pub async fn auto_login( state: Arc<Mutex<Ubisoft>> ) {
        loop {
            state
                .lock().await
                .login().await.expect("Failed to log in!");

            sleep(Duration::from_secs(6300)).await;
        }
    }

    pub async fn basic_request ( &mut self, url: String ) -> Result<Value, Box<dyn Error>> {
        let client = reqwest::Client::new();

        let request = client.get(&url)
            .headers(self.headers.clone());
        
        match 
            request
                .send()
                .await?
                .error_for_status() 
        {
            Ok(response) => {
                Ok(serde_json::from_str(&response.text().await?)?)
            },
            Err(err) => {
                println!("Request to {url} may have failed for reason {err}");
                Err(Box::new(err))
            }
        }
    }
    pub async fn get_account_id ( &mut self, account_id: String, platform: String ) -> Option<String> {
        if account_id.len() < 20 {
            let result = self
                .basic_request(
                    format!("https://public-ubiservices.ubi.com/v3/profiles?nameOnPlatform={}&platformType={}", account_id, platform)
                ).await.ok()?;
                
            return result.get("profiles")
                .and_then(|val| {
                    val.get(0)
                        .and_then(|val| {
                            val.get("userId")
                                .and_then(|val| {
                                    val.as_str()
                                        .and_then(|st| Some(String::from(st)))
                                })
                        })
                });
        }
        Some(account_id)
    }
}