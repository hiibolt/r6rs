use serde::{Deserialize, Serialize};
use anyhow::{ Result, Context };

#[derive(Debug)]
pub struct Database {
    api_key: String
}
#[derive(Debug, Serialize, Deserialize)]
pub struct CommandEntry {
    pub message_id: u64,
    pub user_id:    u64,
    pub server_id:  u64,
    pub command:    String,
    pub result:     String,
}
impl Database {
    pub fn new(api_key: String) -> Self {
        Database {
            api_key
        }
    }
    pub fn verify_db ( &self ) -> Result<()> {
        let database_base_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let table_id = std::env::var("COMMAND_TABLE_ID")
            .expect("COMMAND_TABLE_ID must be set");

        let url = format!("{}/api/v2/tables/{}/records", database_base_url, table_id);

        // Use the `ureq` crate to send a POST request to the database
        let _ = ureq::get(&url)
            .set("xc-token", &self.api_key)
            .call()
            .expect("Failed to send the request!");

        Ok(())
    }
    pub fn upload_command (&self, entry: CommandEntry) -> Result<()> {
        let database_base_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");

        let table_id = std::env::var("COMMAND_TABLE_ID")
            .expect("COMMAND_TABLE_ID must be set");

        let url = format!("{}/api/v2/tables/{}/records", database_base_url, table_id);

        let stringified_entry = serde_json::to_string(&entry)
            .context("Failed to serialize the entry!")?;

        // Use the `ureq` crate to send a POST request to the database
        let _ = ureq::post(&url)
            .set("xc-token", &self.api_key)
            .set("Content-Type", "application/json")
            .send_string(&stringified_entry)
            .expect("Failed to send the request!");

        Ok(())
    }
}