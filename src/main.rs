mod helper;
mod sections;
mod apis;

use crate::{
    helper::lib::{send_embed, inject_documentation},
    apis::{Snusbase, BulkVS, Ubisoft, Database},
    helper::{bot::{Bot, State}, startup::build_root_command, bot::BackendHandles},
};

use std::{
    collections::VecDeque, 
    env, 
    fs::read_to_string, 
    sync::Arc
};

use serde_json::Value;
use serenity::prelude::*;
use serenity::all::{ActivityData, ActivityType, OnlineStatus};
use serenity::model::channel::Message;
use url::Url;
use anyhow::{Result, Context};
use colored::Colorize;

#[tokio::main]
async fn main() -> Result<()> {
    // Get intents and token
    let token = env::var("DISCORD_BOT_TOKEN")
        .context("Expected a token in the environment")?;
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Read data files
    let bot_data_contents: String = read_to_string("assets/bot_data.json")
        .context("Could not find 'assets/bot_data.json', please ensure you have created one!")?;
    let id_list_contents: String = read_to_string("assets/ids.json")
        .context("Could not find 'assets/ids.json', please ensure you have created one!")?;
    let market_data_contents: String = read_to_string("assets/data.json")
        .context("Could not find 'assets/data.json', please ensure you have created one!")?;
    
    // Build the state
    let state = Arc::new(Mutex::new(State {
        bot_data: serde_json::from_str(&bot_data_contents)
            .context("Could not parse the contents of 'bot_data.json'!")?,
        id_list: serde_json::from_str(&id_list_contents)
            .context("Could not parse the contents of 'ids.json'!")?,
        market_data: serde_json::from_str(&market_data_contents)
            .context("Could not parse the contents of 'data.json'!")?,
    }));

    // Build the Snusbase API
    let snusbase = Arc::new(Mutex::new(Snusbase::new()
        .context("Could not create Snusbase API!")?
    ));

    // Build the BulkVS API
    let bulkvs = Arc::new(Mutex::new(BulkVS::new()
        .context("Could not create BulkVS API!")?
    ));

    // Build the Ubisoft API and log in
    let ubisoft_api = Arc::new(Mutex::new(Ubisoft::new(
    env::var("UBISOFT_AUTH_EMAIL")
        .context("Could not find UBISOFT_AUTH_EMAIL in the environment!")?,
    env::var("UBISOFT_AUTH_PW")
        .context("Could not find UBISOFT_AUTH_PW in the environment!")?
    )));

    // Build the Database object and log in
    let database = Arc::new(Mutex::new(Database::new(
        env::var("DATABASE_API_KEY")
            .context("Could not find DATABASE_API_KEY in the environment!")?
    )));

    // Test that the database is operational
    if let Err(e) = database
        .lock().await
        .verify_db() {
        warn!("Failed to update DB with reason `{e}`!");
    }

    // Start login process
    tokio::spawn(Ubisoft::auto_login( ubisoft_api.clone()));

    // Start autosave
    tokio::spawn(helper::lib::autosave( state.clone() ));

    // Start autopull
    tokio::spawn(helper::lib::autopull( state.clone() ));

    // Build the root command
    let root_command = Arc::new(Mutex::new(build_root_command().await));

    // Write command documentation
    inject_documentation(
        &root_command
            .lock().await
            .print_help(
                String::from(""), 
                2, 
                true
            ).await
    ).await?;

    // Build client with state
    let mut client =
        Client::builder(&token, intents)
        .event_handler(Bot {
            root_command,

            backend_handles: BackendHandles {
                ubisoft_api,
                snusbase,
                bulkvs,
                database,
                state
            }
        })
        .activity(ActivityData {
            name: String::from("serverspace"),
            kind: ActivityType::Competing,
            state: Some(String::from("Written and maintained by @hiibolt")),
            url: Some(Url::parse("https://github.com/hiibolt/")
                .context("Hardcoded URL is invalid!")?
            )
        })
        .status(OnlineStatus::DoNotDisturb)
        .await
        .context("Err creating client")?;
    
    // Start r6rs
    client.start().await?;

    Ok(())
}