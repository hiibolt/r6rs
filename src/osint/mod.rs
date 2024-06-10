use anyhow::{ Result, Context, bail };
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serenity::all::{CreateAttachment, CreateMessage, Message};
use tokio::sync::Mutex;
use std::{collections::{HashMap, VecDeque}, fmt::{self, Display, Formatter}, sync::Arc};

use crate::helper::send_embed;

#[derive(Debug, Clone, Deserialize)]
pub struct BulkVSPhoneNumberResponse {
    pub name: Option<String>,
    pub number: Option<String>,
    pub time: Option<i64>
}
#[derive(Debug)]
pub struct BulkVS {
    api_key: String,
}
impl BulkVS {
    pub fn new () -> Result<Self> {
        Ok(Self {
            api_key: std::env::var("BULKVS_API_KEY")
                .context("Couldn't find API key in environment! Be sure to set `BULKVS_API_KEY`.")?
        })
    }
    pub fn query_phone_number ( &self, phone_number: &str ) -> Result<BulkVSPhoneNumberResponse> {
        let path = format!(
            "https://cnam.bulkvs.com/?id={}&did={}&format=json",
            self.api_key,
            phone_number);

        let resp_object = ureq::get(&path)
            .call()
            .context("Failed to query Snusbase!")?;

        let resp_object_string = resp_object.into_string()
            .context("Failed to convert response into string!")?;

        let resp_deserialized = serde_json::from_str(&resp_object_string)
            .context("Failed to deserialize response!")?;

        Ok(resp_deserialized)
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SnusbaseResponse {
    took: i32,
    size: i32,
    results: HashMap<String, Vec<HashMap<String, String>>>
}
#[derive(Debug, Serialize, Deserialize)]
pub struct SnusbaseIpResponse {
    took: i32,
    size: i32,
    results: HashMap<String, HashMap<String, Value>>
}
impl Display for SnusbaseResponse {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for (dump, content) in &self.results {
            for entry in content {
                write!(f, "## Dump (*{}*):\n", dump)?;

                for (key, value) in entry {
                    write!(f, "- **{}**: {}\n", key, value)?;
                }
                
                write!(f, "\n")?;
            }
        }
        
        Ok(())
    }
}
impl SnusbaseResponse {
    pub fn _dumps ( &self ) -> Vec<String> {
        self.results
            .keys()
            .map(|key| key.to_string())
            .collect()
    }
    pub fn _usernames ( &self ) -> Vec<String> {
        let mut usernames = Vec::new();

        for (_dump, content) in &self.results {
            for entry in content {
                if let Some(username) = entry.get("username") {
                    usernames.push(username.to_string());
                }
            }
        }

        usernames    
    }
    pub fn _emails ( &self ) -> Vec<String> {
        let mut emails = Vec::new();

        for (_dump, content) in &self.results {
            for entry in content {
                if let Some(email) = entry.get("email") {
                    emails.push(email.to_string());
                }
            }
        }

        emails
    }
    pub fn _passwords ( &self ) -> Vec<String> {
        let mut passwords = Vec::new();

        for (_dump, content) in &self.results {
            for entry in content {
                if let Some(password) = entry.get("password") {
                    passwords.push(password.to_string());
                }
            }
        }

        passwords
    }
    pub fn _names ( &self ) -> Vec<String> {
        let mut names = Vec::new();

        for (_dump, content) in &self.results {
            for entry in content {
                if let Some(name) = entry.get("name") {
                    names.push(name.to_string());
                }
            }
        }

        names
    }
    pub fn _last_ips ( &self ) -> Vec<String> {
        let mut last_ips = Vec::new();

        for (_dump, content) in &self.results {
            for entry in content {
                if let Some(last_ip) = entry.get("last_ip") {
                    last_ips.push(last_ip.to_string());
                }
            }
        }

        last_ips
    }
    pub fn _addresses ( &self ) -> Vec<String> {
        let mut addresses = Vec::new();

        for (_dump, content) in &self.results {
            for entry in content {
                if let Some(address) = entry.get("address") {
                    addresses.push(address.to_string());
                }
            }
        }

        addresses
    }
    pub fn _companies ( &self ) -> Vec<String> {
        let mut companies = Vec::new();

        for (_dump, content) in &self.results {
            for entry in content {
                if let Some(company) = entry.get("company") {
                    companies.push(company.to_string());
                }
            }
        }

        companies
    }
}
#[derive(Debug)]
pub struct Snusbase {
    api_key: String,
}
impl Snusbase {
    pub fn new() -> Result<Self> {
        Ok(Self { 
            api_key: std::env::var("SNUSBASE_API_KEY")
                .context("Couldn't initialize Snusbase client")? 
        })
    }
    pub async fn whois_ip_query (
        &self,
        ips: Vec<String>
    ) -> Result<SnusbaseIpResponse> {
        if ips.len() == 0 {
            bail!("No IPs to query!");
        }
        
        // Query Snusbase
        let resp_object = ureq::post("https://api-experimental.snusbase.com/tools/ip-whois")
            .set("Auth", &self.api_key )
            .set("Content-Type", "application/json")
            .send_json(ureq::json!({
                "terms": ips
            }))
            .context("Failed to query Snusbase!")?;

        // Debug print response
        let resp_as_string = resp_object.into_string()
            .context("Failed to convert response to string!")?;
        
        // Deserialize response with serde_json
        let deserialized_resp: SnusbaseIpResponse = serde_json::from_str(&resp_as_string)
            .context("Failed to deserialize response!")?;
        
        Ok(deserialized_resp)
    }
    pub async fn database_query ( 
        &self,
        terms: Vec<String>,
        types: Vec<String>,
        wildcard: bool
    ) -> Result<SnusbaseResponse> {
        // Query Snusbase
        let resp_object = ureq::post("https://api-experimental.snusbase.com/data/search")
            .set("Auth", &self.api_key )
            .set("Content-Type", "application/json")
            .send_json(ureq::json!({
                "terms": terms,
                "types": types,
                "wildcard": wildcard
            }))
            .context("Failed to query Snusbase!")?;

        // Debug print response
        let resp_as_string = resp_object.into_string()
            .context("Failed to convert response to string!")?;
        
        // Deserialize response with serde_json
        let deserialized_resp: SnusbaseResponse = serde_json::from_str(&resp_as_string)
            .context("Failed to deserialize response!")?;
        
        Ok(deserialized_resp)
    }
    pub async fn get_by_email (
        &self,
        email: String
    ) -> Result<SnusbaseResponse> {
        self.database_query(
            vec!(email),
            vec!(String::from("email")),
            false
        ).await
    }
    pub async fn get_by_username (
        &self,
        username: String
    ) -> Result<SnusbaseResponse> {
        self.database_query(
            vec!(username),
            vec!(String::from("username")),
            false
        ).await
    }
    pub async fn get_by_last_ip (
        &self,
        last_ip: String
    ) -> Result<SnusbaseResponse> {
        self.database_query(
            vec!(last_ip),
            vec!(String::from("lastip")),
            false
        ).await
    }
    pub async fn get_by_password (
        &self,
        password: String
    ) -> Result<SnusbaseResponse> {
        self.database_query(
            vec!(password),
            vec!(String::from("password")),
            false
        ).await
    }
    pub async fn get_by_name (
        &self,
        name: String
    ) -> Result<SnusbaseResponse> {
        self.database_query(
            vec!(name),
            vec!(String::from("name")),
            false
        ).await
    }
}
pub async fn lookup( 
    snusbase: Arc<Mutex<Snusbase>>,
    ctx: serenity::client::Context,
    msg: Message,
    mut args: VecDeque<String>,
    lookup_type: &str
) {
    let snusbase_response_result = match lookup_type {
        "email" => {
            let mut ret = Err(anyhow::anyhow!("No email provided!"));

            if let Some(email) = args.pop_front() {
                ret = snusbase.lock()
                    .await
                    .get_by_email(email)
                    .await;
            }
            
            ret
        },
        "username" => {
            let mut ret = Err(anyhow::anyhow!("No username provided!"));

            if let Some(username) = args.pop_front() {
                ret = snusbase.lock()
                    .await
                    .get_by_username(username)
                    .await;
            }
            
            ret
        },
        "password" => {
            let mut ret = Err(anyhow::anyhow!("No password provided!"));

            if let Some(password) = args.pop_front() {
                ret = snusbase.lock()
                    .await
                    .get_by_password(password)
                    .await;
            }
            
            ret
        },
        "last_ip" => {
            let mut ret = Err(anyhow::anyhow!("No last IP provided!"));

            if let Some(last_ip) = args.pop_front() {
                ret = snusbase.lock()
                    .await
                    .get_by_last_ip(last_ip)
                    .await;
            }

            ret
        }
        "name" => {
            let mut ret = Err(anyhow::anyhow!("No name provided!"));

            if args.len() > 0 {
                ret = snusbase.lock()
                    .await
                    .get_by_name(args.into_iter().collect::<Vec<String>>().join(" "))
                    .await;
            }
            
            ret
        },
        _ => { panic!("Invalid lookup type!"); }
    };

    if snusbase_response_result.is_err() {
        send_embed(
            &ctx, 
            &msg, 
            "An error occured", 
            &format!("{}", snusbase_response_result.unwrap_err()), 
            "https://github.com/hiibolt/hiibolt/assets/91273156/831e2922-cdcb-409d-a919-1a72fbe56ff4"
        ).await
            .unwrap();

        return;
    }

    let snusbase_response = snusbase_response_result.expect("unreachable");

    if snusbase_response.results.len() > 10 {
        let full_dump = format!("{}", snusbase_response);

        send_embed(
            &ctx, 
            &msg, 
            "OSINT DUMP", 
            "There were more than 10 results, which in total contains more data than Discord can display.\n\nA full dump will be attached below shortly!", 
            "https://github.com/hiibolt/hiibolt/assets/91273156/831e2922-cdcb-409d-a919-1a72fbe56ff4"
        ).await
            .unwrap();

        let builder = CreateMessage::new();

        msg.channel_id.send_files(
            &ctx.http,
            std::iter::once(CreateAttachment::bytes(
                full_dump.as_bytes(),
                "full_dump.txt"
            )),
            builder
        ).await
            .unwrap();

        return;
    }

    if snusbase_response.results.len() == 0 {
        send_embed(
            &ctx, 
            &msg, 
            "No results", 
            "Nothing were found for the given query!\n\nThere were no errors, but there weren't any results either.", 
            "https://github.com/hiibolt/hiibolt/assets/91273156/831e2922-cdcb-409d-a919-1a72fbe56ff4"
            ).await
                .unwrap();
        
        return;
    }

    let len = snusbase_response.results.len();
    for (ind, (dump, content)) in snusbase_response.results.iter().take(10).enumerate() {
        for entry in content {
            let mut message = String::new();
            
            message += &format!("## Dump {}/{len}\n(From `{}`):\n", ind + 1, dump);

            for (key, value) in entry {
                message += &format!("- **{}**: {}\n", key, value);
            }
            
            message += "\n";

            send_embed(
                &ctx, 
                &msg, 
                "OSINT DUMP - Via Email", 
                &message, 
                "https://github.com/hiibolt/hiibolt/assets/91273156/831e2922-cdcb-409d-a919-1a72fbe56ff4"
            ).await
                .unwrap();
        }
    }
}
pub async fn help (
    ctx: serenity::client::Context,
    msg: Message
) {
    send_embed(
        &ctx, 
        &msg, 
        "OSINT Help", 
        "The `osint` command is used to query for information on emails, usernames, IPs, passwords and names.\n\n**Subcommands**:\n- `email` - Query by email\n- `username` - Query by username\n- `last_ip` Query by IP\n- `password` - Query by password\n- `name` - Query by name\n- `ip` - Geolocate by IP\n- `phone` - Perform CNAM lookup\n\n**Usage**:\n- `>>osint email <email>`\n- `>>osint username <username>`\n- `>>osint ip <ip>`\n- `>>osint password <password>`\n- `>>osint name <name>`\n- `>>osint last_ip <last ip>`\n- `>>osint phone <phone number>`", 
        "https://github.com/hiibolt/hiibolt/assets/91273156/831e2922-cdcb-409d-a919-1a72fbe56ff4"
            ).await
                .unwrap();
}
pub async fn osint ( 
    snusbase: Arc<Mutex<Snusbase>>,
    bulkvs: Arc<Mutex<BulkVS>>,
    ctx: serenity::client::Context,
    msg: Message,
    mut args: VecDeque<String> 
) {
    match args
        .pop_front()
        .unwrap_or(String::from("help"))
        .as_str()
    {
        "email" => {
            tokio::spawn(lookup(snusbase, ctx, msg, args, "email"));
        },
        "username" => {
            tokio::spawn(lookup(snusbase, ctx, msg, args, "username"));
        },
        "last_ip" => {
            tokio::spawn(lookup(snusbase, ctx, msg, args, "last_ip"));
        },
        "phone" => {
            if let Some(phone_number) = args.pop_front() {
                let response = bulkvs.lock()
                    .await
                    .query_phone_number(&phone_number);

                if response.is_err() {
                    send_embed(
                        &ctx, 
                        &msg, 
                        "An error occured", 
                        &format!("{}", response.unwrap_err()), 
                        "https://github.com/hiibolt/hiibolt/assets/91273156/831e2922-cdcb-409d-a919-1a72fbe56ff4"
                    ).await
                        .unwrap();
    
                    return;
                }

                let response = response.unwrap();
    
                let mut message = String::new();
                if let Some(name) = response.name {
                    message += &format!("\n- **Name**: {name}");
                }
                if let Some(number) = response.number {
                    message += &format!("\n- **Number**: {number}");
                }
                if let Some(time) = response.time {
                    message += &format!("\n- **Time**: {time}");
                }

                send_embed(
                    &ctx, 
                    &msg, 
                    "CNAM Lookup", 
                    &message, 
                    "https://github.com/hiibolt/hiibolt/assets/91273156/831e2922-cdcb-409d-a919-1a72fbe56ff4"
                ).await
                    .unwrap();

                return;
            }
            
            send_embed(
                &ctx, 
                &msg, 
                "An error occured", 
                "Missing phone number!", 
                "https://github.com/hiibolt/hiibolt/assets/91273156/831e2922-cdcb-409d-a919-1a72fbe56ff4"
            ).await
                .unwrap();
        },
        "ip" => {
            let response = snusbase.lock()
                .await
                .whois_ip_query(args.into_iter().collect())
                .await;

            if response.is_err() {
                send_embed(
                    &ctx, 
                    &msg, 
                    "An error occured", 
                    &format!("{}", response.unwrap_err()), 
                    "https://github.com/hiibolt/hiibolt/assets/91273156/831e2922-cdcb-409d-a919-1a72fbe56ff4"
                ).await
                    .unwrap();

                return;
            }

            let response = response.unwrap();

            let mut message = String::new();
            for (ip, content) in &response.results {
                message += &format!("## IP (*{}*):\n", ip);

                for (key, value) in content {
                    if value.is_string() {
                        message += &format!("\n- **{}**: {:?}", key, value.as_str().unwrap());
                    } else if value.is_number() {
                        message += &format!("\n- **{}**: {:?}", key, value.as_number().unwrap());
                    } else {
                        message += &format!("\n- **{}**: {:?}", key, value);
                    }
                }
            }

            send_embed(
                &ctx, 
                &msg, 
                "IP Lookup", 
                &message, 
                "https://github.com/hiibolt/hiibolt/assets/91273156/831e2922-cdcb-409d-a919-1a72fbe56ff4"
            ).await
                .unwrap();
        },
        "password" => {
            tokio::spawn(lookup(snusbase, ctx, msg, args, "password"));
        },
        "name" => {
            tokio::spawn(lookup(snusbase, ctx, msg, args, "name"));
        },
        "help" => {
            tokio::spawn(help( ctx, msg ));
        },
        nonexistant => {
            send_embed(
                &ctx, 
                &msg, 
                "Command does not exist", 
                &format!("The subcommand `{nonexistant}` is not valid!\n\nConfused?\nRun `osint help` for information on `osint`'s commands\nRun `r6 help` for information on all commands"), 
                "https://github.com/hiibolt/hiibolt/assets/91273156/831e2922-cdcb-409d-a919-1a72fbe56ff4"
            ).await
                .unwrap();
        }
    }
}