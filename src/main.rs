mod opsec;
mod admin;
mod bans;
mod econ;
mod help;
mod helper;
mod auth;
mod commands;
mod snusbase;

use crate::{
    econ::econ,
    opsec::opsec,
    bans::bans,
    admin::admin,
    help::help,
    helper::{ no_access, unimplemented, send_embed },
    auth::UbisoftAPI
};

use std::{
    env,
    collections::VecDeque,
    fs::read_to_string,
    sync::Arc
};

use snusbase::{osint, Snusbase};
use tokio::sync::Mutex;
use serde_json::Value;
use serenity::{all::{ActivityData, ActivityType, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId, Interaction, OnlineStatus}, async_trait};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use url::Url;
use std::collections::HashMap;

#[derive(Debug)]
struct State {
    bot_data: Value,
    id_list: HashMap<String, String>,
    market_data: Value
}

#[derive(Debug)]
struct Bot {
    ubisoft_api: Arc<Mutex<UbisoftAPI>>,
    snusbase:    Arc<Mutex<Snusbase>>,
    state:       Arc<Mutex<State>>
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut args: VecDeque<String> = msg.content
            .clone()
            .split(' ')
            .map(|i| String::from(i))
            .collect();
        let user_id: u64 = msg.author.id.get();

        if args.pop_front() != Some(String::from("r6")) {
            return;
        }

        match args
            .pop_front()
            .unwrap_or(String::from("help"))
            .as_str()
        {
            "econ" => {
                // Check if they're not on the whitelist
                if !self.state
                    .lock().await
                    .bot_data["whitelisted_user_ids"]["econ"]
                    .as_array().expect("The user id whitelists must be lists, even if it's 0-1 users!")
                    .iter()
                    .any(|x| x.as_u64().expect("User ids need to be numbers!") == user_id)
                {
                    no_access( ctx, msg.clone(), "econ", user_id ).await;
                    return;
                }

                // Otherwise, go ahead
                tokio::spawn(econ(self.state.clone(), ctx, msg, args));
            },
            "opsec" => {
                // Check if they're not on the whitelist
                if !self.state
                    .lock().await
                    .bot_data["whitelisted_user_ids"]["opsec"]
                    .as_array().expect("The user id whitelists must be lists, even if it's 0-1 users!")
                    .iter()
                    .any(|x| x.as_u64().expect("User ids need to be numbers!") == user_id)
                {
                    no_access( ctx, msg.clone(), "opsec", user_id ).await;
                    return;
                }

                // Otherwise, go ahead
                tokio::spawn(opsec(self.ubisoft_api.clone(), ctx, msg, args)); 
            },
            "osint" => {
                // Check if they're not on the whitelist
                if !self.state
                    .lock().await
                    .bot_data["whitelisted_user_ids"]["osint"]
                    .as_array().expect("The user id whitelists must be lists, even if it's 0-1 users!")
                    .iter()
                    .any(|x| x.as_u64().expect("User ids need to be numbers!") == user_id)
                {
                    no_access( ctx, msg.clone(), "osint", user_id ).await;
                    return;
                }

                // Otherwise, go ahead
                tokio::spawn(osint(self.snusbase.clone(), ctx, msg, args)); 
            }
            "bans" => {
                // Check if they're not on the whitelist
                if !self.state
                    .lock().await
                    .bot_data["whitelisted_user_ids"]["bans"]
                    .as_array().expect("The user id whitelists must be lists, even if it's 0-1 users!")
                    .iter()
                    .any(|x| x.as_u64().expect("User ids need to be numbers!") == user_id)
                {
                    no_access( ctx, msg.clone(), "bans", user_id ).await;
                    return;
                }

                // Otherwise, go ahead
                tokio::spawn(bans(ctx, msg, args));
            },
            "admin" => {
                // Check if they're not on the whitelist
                if !self.state
                    .lock().await
                    .bot_data["whitelisted_user_ids"]["admin"]
                    .as_array().expect("The user id whitelists must be lists, even if it's 0-1 users!")
                    .iter()
                    .any(|x| x.as_u64().expect("User ids need to be numbers!") == user_id)
                {
                    no_access( ctx, msg.clone(), "admin", user_id ).await;
                    return;
                }

                // Otherwise, go ahead
                tokio::spawn(admin( self.state.clone(), ctx, msg, args ));
            },
            _ => { tokio::spawn(help(ctx, msg)); }
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            println!("Received command interaction: {}", command.data.name);

            let content = match command.data.name.as_str() {
                "announce" => {
                    let response = commands::announce_all::run(
                            command.data.options(), 
                            &ctx, 
                            self.state.clone()
                        ).await.expect("Failed to run command!");
                
                    Some(response)
                },
                "announce_opsec" => {
                    let response = commands::announce_opsec::run(
                            command.data.options(), 
                            &ctx, 
                            self.state.clone()
                        ).await.expect("Failed to run command!");
                
                    Some(response)
                },
                "announce_econ" => {
                    let response = commands::announce_econ::run(
                            command.data.options(), 
                            &ctx, 
                            self.state.clone()
                        ).await.expect("Failed to run command!");
                
                    Some(response)
                },
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    println!("Cannot respond to slash command: {why}");
                }
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected with data!", ready.user.name);

        let guild_id = GuildId::new(
            env::var("GUILD_ID")
                .expect("Expected GUILD_ID in environment")
                .parse()
                .expect("GUILD_ID must be an integer"),
        );

        let commands = guild_id
            .set_commands(&ctx.http, vec![
                commands::announce_all::register(),
                commands::announce_opsec::register(),
                commands::announce_econ::register(),
            ])
            .await.expect("Failed to register guild commands!");

        let command_names = commands
            .iter()
            .map(|x| x.name.clone())
            .collect::<Vec<String>>();
        println!("I now have the following guild slash commands: {command_names:?}");
    }
}

#[tokio::main]
async fn main() {
    // Get intents and token
    let token = env::var("DISCORD_BOT_TOKEN").expect("Expected a token in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Read data files
    let bot_data_contents: String = read_to_string("assets/bot_data.json")
        .expect("Could not find 'assets/bot_data.json', please ensure you have created one!");
    let id_list_contents: String = read_to_string("assets/ids.json")
        .expect("Could not find 'assets/ids.json', please ensure you have created one!");
    let market_data_contents: String = read_to_string("assets/data.json")
        .expect("Could not find 'assets/data.json', please ensure you have created one!");
    
    // Build the state into an async mutex
    let state = Arc::new(Mutex::new(
        State {
            bot_data: serde_json::from_str(&bot_data_contents)
                .expect("Could not parse the contents of 'bot_data.json'!"),
            id_list: serde_json::from_str(&id_list_contents)
                .expect("Could not parse the contents of 'ids.json'!"),
            market_data: serde_json::from_str(&market_data_contents)
                .expect("Could not parse the contents of 'data.json'!"),
        }
    ));

    // Build the Snusbase API
    let snusbase = Arc::new(
        Mutex::new(
            Snusbase::new().expect("Could not create Snusbase API!")
        )
    );

    // Build the Ubisoft API and log in
    let ubisoft_api = Arc::new(
        Mutex::new(
            UbisoftAPI::new(
                env::var("UBISOFT_AUTH_EMAIL")
                    .expect("Could not find UBISOFT_AUTH_EMAIL in the environment!"),
                env::var("UBISOFT_AUTH_PW")
                    .expect("Could not find UBISOFT_AUTH_PW in the environment!")
            )
        )
    );

    // Start login process
    tokio::spawn(UbisoftAPI::auto_login( ubisoft_api.clone()));

    // Start autosave
    tokio::spawn(helper::autosave( state.clone() ));

    // Start autopull
    tokio::spawn(helper::autopull( state.clone() ));

    // Build client with state
    let mut client =
        Client::builder(&token, intents)
        .event_handler(Bot {
            snusbase,
            ubisoft_api,
            state: state
        })
        .activity(ActivityData {
            name: String::from("Rainbow Six Siege"),
            kind: ActivityType::Competing,
            state: Some(String::from("Powered by Rust and Serenity.")),
            url: Some(Url::parse("https://github.com/hiibolt/").expect("Hardcoded URL is invalid!"))
        })
        .status(OnlineStatus::DoNotDisturb)
        .await.expect("Err creating client");
    
    // Start r6rs
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}