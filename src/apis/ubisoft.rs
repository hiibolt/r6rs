use crate::{
    error, info, Value, Arc, Mutex, 
};

use colored::Colorize;
use serde::{Deserialize, Serialize};
use base64::prelude::*;
use anyhow::{Result, bail, anyhow, Context};
use reqwest::{header::HeaderMap, StatusCode};
use tokio::{
    fs::read_to_string, 
    time::{sleep, Duration}
};

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
    pub async fn login ( &mut self ) -> Result<()> {
        let proxy_url = std::env::var("PROXY_URL")
            .context("Couldn't find proxy URL in environment! Be sure to set `PROXY_URL`.")?;
        let proxy = reqwest::Proxy::all(proxy_url)
            .context("Failed to create proxy!")?;
        let client = reqwest::Client::builder()
            .proxy(proxy)
            .build()
            .context("Failed to create HTTP client!")?;

        let auth_header = format!("Basic {}", self.token);
        println!("Using auth header: {}", auth_header.green());

        self.headers.insert("Authorization", auth_header.parse()?);
        self.headers.insert("User-Agent", "UbiServices_SDK_2020.Release.58_PC64_ansi_static".parse()?);
        self.headers.insert("Content-Type", "application/json; charset=UTF-8".parse()?);
        self.headers.insert("Ubi-AppId", "4391c956-8943-48eb-8859-07b0778f47b9".parse()?);
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

                self.headers.insert("Authorization", format!("Ubi_v1 t={}", response_json["ticket"].as_str().ok_or(anyhow!("Ticket missing from Ubi response!"))?).parse()?);
                self.headers.insert("Ubi-SessionId", response_json["sessionId"].as_str().ok_or(anyhow!("Ticket missing from Ubi response!"))?.parse()?);
    
                info!("Successfully authenticated!");
            },
            _ => {
                error!("Failed to authenticate! Response: \"{response:#?}\"");

                bail!("Failed to authenticate with given login! Verify your information is correct.");
            }
        }

        Ok(())
    }
    pub async fn auto_login( state: Arc<Mutex<Ubisoft>> ) {
        loop {
            info!("Reauthenticating with Ubisoft!");

            state
                .lock().await
                .login().await.expect("Failed to log in!");

            sleep(Duration::from_secs(6300)).await;
        }
    }

    pub async fn basic_request ( &mut self, url: String ) -> Result<Value> {
        let proxy_url = std::env::var("PROXY_URL")
            .context("Couldn't find proxy URL in environment! Be sure to set `PROXY_URL`.")?;
        let proxy = reqwest::Proxy::all(proxy_url)
            .context("Failed to create proxy!")?;
        let client = reqwest::Client::builder()
            .proxy(proxy)
            .build()
            .context("Failed to create HTTP client!")?;

        let request = client.get(&url)
            .headers(self.headers.clone());
        
        match 
            request
                .send()
                .await?
                .error_for_status() 
        {
            Ok(response) => {
                let json = serde_json::from_str(
                        &response.text().await
                            .context("Failed to extract text for basic request!")?
                    )
                    .context("Failed to unwrap JSON for basic request!")?;

                Ok(json)
            },
            Err(err) => {
                bail!("Request to {url} may have failed for reason {:#?}", err);
            }
        }
    }
    pub async fn graphql_request ( &mut self, url: String, body: String ) -> Result<Value> {
        let proxy_url = std::env::var("PROXY_URL")
            .context("Couldn't find proxy URL in environment! Be sure to set `PROXY_URL`.")?;
        let proxy = reqwest::Proxy::all(proxy_url)
            .context("Failed to create proxy!")?;
        let client = reqwest::Client::builder()
            .proxy(proxy)
            .build()
            .context("Failed to create HTTP client!")?;

        let mut headers_with_new_locale = self.headers.clone();
        headers_with_new_locale.insert("Ubi-LocaleCode", "en-US".parse()?);

        let request = client.post(&url)
            .headers(headers_with_new_locale)
            .body(body);

        
        match 
            request
                .send()
                .await?
                .error_for_status() 
        {
            Ok(response) => {
                let json = serde_json::from_str(
                        &response.text().await
                            .context("Failed to extract text for basic request!")?
                    )
                    .context("Failed to unwrap JSON for basic request!")?;

                Ok(json)
            },
            Err(err) => {
                bail!("Request to {url} may have failed for reason {err}");
            }
        }
    }
    pub async fn get_applications ( &mut self, account_id: String ) -> Result<Value> {
        self.basic_request(format!("https://public-ubiservices.ubi.com/v2/profiles/applications?profileIds={account_id}&spaceIds=45d58365-547f-4b45-ab5b-53ed14cc79ed"))
            .await
            .map_err(|err| anyhow!("{:?}", err))
    }
    pub async fn get_account_id ( &mut self, account_id: String, platform: String ) -> Result<String> {
        if account_id.len() < 20 {
            let result = self
                .basic_request(
                    format!("https://public-ubiservices.ubi.com/v3/profiles?nameOnPlatform={}&platformType={}", account_id, platform)
                ).await
                .context("Failed to ask Ubi for profile ID!")?;
                
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
                })
                .ok_or(anyhow!("Couldn't locate ID in response!"));
        }
        Ok(account_id)
    }

    pub async fn get_least_sold ( 
        &mut self,
        number_of_items: usize
    ) -> Result<Vec<DisplayableItem>> {
        info!("Attempting GraphQL request...");
    
        // Load the `query.txt` file
        let path = "assets/query.txt";
        let query = read_to_string(path)
            .await
            .context("Could not find 'assets/query.txt', please ensure you have created one!")?;
    
        let mut offset: usize = 0;
        let mut items: Vec<DisplayableItem> = Vec::new();
    
        while items.len() < number_of_items {
            info!("Passing with offset: {offset}");
    
            let lowest_sales_raw: Value = self
                .graphql_request(
                    format!("https://public-ubiservices.ubi.com/v1/profiles/me/uplay/graphql"),
                    query.replace("PLACEHOLDER_OFFSET_REPLACEME", &offset.to_string())
                )
                .await.expect("Failed to get lowest sales!");
    
            let lowest_sales: Vec<UbisoftGraphQLResponse> = serde_json::from_value(lowest_sales_raw)
                .context("Failed to parse lowest sales!")?;
    
            for node in lowest_sales[0].data.game.marketable_items.nodes.iter() {
                if let Some(item) = unpack_node(node.clone()) {
                    if item.last_sold_at > 180 {
                        continue;
                    }
    
                    items.push(item);
    
                    if items.len() >= number_of_items {
                        break;
                    }
                }
            }
    
            offset += 40;
        }
    
        Ok(items)
    }
    pub async fn get_least_sold_owned ( 
        &mut self,
        number_of_items: usize
    ) -> Result<Vec<DisplayableItem>> {
        info!("Attempting GraphQL request...");
    
        // Load the `query.txt` file
        let path = "assets/query_owned.txt";
        let query = read_to_string(path)
            .await
            .context("Could not find 'assets/query_owned.txt', please ensure you have created one!")?;
    
        let mut offset: usize = 0;
        let mut items: Vec<DisplayableItem> = Vec::new();
    
        while items.len() < number_of_items {
            info!("Passing with offset: {offset}");
    
            let lowest_sales_raw: Value = self
                .graphql_request(
                    format!("https://public-ubiservices.ubi.com/v1/profiles/me/uplay/graphql"),
                    query.replace("PLACEHOLDER_OFFSET_REPLACEME", &offset.to_string())
                )
                .await.expect("Failed to get lowest sales!");
    
            let lowest_sales: Vec<UbisoftGraphQLResponseWithViewer> = serde_json::from_value(lowest_sales_raw.clone())
                .map_err(|e| anyhow!("Failed to parse lowest sales!\\n\\n{e:#?}\\n\\nRaw Response: {:?}", lowest_sales_raw.to_string().chars().take(300).collect::<String>()))?;
    
            for node in lowest_sales[0].data.game.viewer.meta.marketable_items.nodes.iter() {
                if let Some(item) = unpack_node(node.clone()) {
                    if item.last_sold_at > 180 {
                        continue;
                    }
    
                    items.push(item);
    
                    if items.len() >= number_of_items {
                        break;
                    }
                }
            }
    
            offset += 40;
        }
    
        Ok(items)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct UbisoftGraphQLMarketableItems {
    nodes: Vec<Value>,
    #[serde(rename = "totalCount")]
    total_count: i32
}
#[derive(Debug, Serialize, Deserialize)]
struct UbisoftGraphQLGame {
    id: String,
    #[serde(rename = "marketableItems")]
    marketable_items: UbisoftGraphQLMarketableItems
}
#[derive(Debug, Serialize, Deserialize)]
struct UbisoftGraphQLData {
    game: UbisoftGraphQLGame
}
#[derive(Debug, Serialize, Deserialize)]
struct UbisoftGraphQLResponse {
    data: UbisoftGraphQLData
}

#[derive(Debug, Serialize, Deserialize)]
struct UbisoftGraphQLDataWithViewer {
    game: UbisoftGraphQLGameWithViewer
}
#[derive(Debug, Serialize, Deserialize)]
struct UbisoftGraphQLGameWithViewer {
    id: String,
    viewer: UbisoftGraphQLViewer
}
#[derive(Debug, Serialize, Deserialize)]
struct UbisoftGraphQLViewer {
    meta: UbisoftGraphQLMeta
}
#[derive(Debug, Serialize, Deserialize)]
struct UbisoftGraphQLMeta {
    id: String,
    #[serde(rename = "marketableItems")]
    marketable_items: UbisoftGraphQLMarketableItems
}
#[derive(Debug, Serialize, Deserialize)]
struct UbisoftGraphQLResponseWithViewer {
    data: UbisoftGraphQLDataWithViewer
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayableItem {
    pub item_id: String,
    pub asset_url: String,
    pub item_type: String,
    pub name: String,

    pub sellers: usize,
    pub last_sold_at: usize
}
fn unpack_node ( node: Value ) -> Option<DisplayableItem> {
    let item_id = node.get("item")?.get("itemId")?.as_str()?.to_string();
    let asset_url = node.get("item")?.get("assetUrl")?.as_str()?.to_string();
    let item_type = format!("{} - {}",
        node.get("item")?.get("type")?.as_str()?.to_string(),
        node.get("item")?.get("tags")?.as_array()?.iter().next()?.as_str()?.to_string());
    let name = node.get("item")?.get("name")?.as_str()?.to_string();

    let sellers = node.get("marketData")?.get("sellStats")?.as_array()?.iter().next()?.get("activeCount")?.as_i64()? as usize;
    let last_sold_at = node.get("marketData")?.get("lastSoldAt")?.as_array()?.iter().next()?.get("price")?.as_i64()? as usize;

    Some(DisplayableItem {
        item_id,
        asset_url,
        item_type,
        name,
        sellers,
        last_sold_at
    })
}
