mod helper;
mod commands;
mod sections;
mod apis;

use crate::{
    sections::{ econ, opsec, admin, osint },
    helper::{ no_access, send_embed, get_random_anime_girl },
    apis::{ Snusbase, BulkVS, Ubisoft, Database }
};

use std::{
    env,
    collections::VecDeque,
    fs::read_to_string,
    sync::Arc
};

use apis::database::CommandEntry;
use tokio::sync::Mutex;
use serde_json::Value;
use serenity::{all::{ActivityData, ActivityType, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId, Interaction, OnlineStatus}, async_trait};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use url::Url;
use std::collections::HashMap;
use anyhow::Result;

#[derive(Debug)]
struct State {
    bot_data: Value,
    id_list: HashMap<String, String>,
    market_data: Value
}

#[derive(Debug)]
struct Bot {
    ubisoft_api: Arc<Mutex<Ubisoft>>,
    snusbase:    Arc<Mutex<Snusbase>>,
    bulkvs:      Arc<Mutex<BulkVS>>,
    state:       Arc<Mutex<State>>,
    database:    Arc<Mutex<Database>>
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(
        &self, 
        ctx: serenity::client::Context, 
        msg: Message
    ) {
        let mut args: VecDeque<String> = msg.content
            .clone()
            .split(' ')
            .map(|i| String::from(i))
            .collect();
        let user_id: u64 = msg.author.id.get();
        let message_id: u64 = msg.id.get();
        let server_id = msg.guild_id
            .and_then(|gid| Some(gid.get()))
            .unwrap_or(0u64);

        let front_arg = args.pop_front().unwrap();

        if &front_arg.chars().take(2).collect::<String>() != ">>" {
            return;
        }


        if let Err(e) = self.database
            .lock().await
            .upload_command(CommandEntry { 
                message_id,
                user_id,
                server_id,
                command: msg.content.clone()
            }) {
            println!("Failed to update DB with reason `{e}`!");
        }

        match front_arg.chars().skip(2).collect::<String>().as_str() {
            "r6" => {
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
                tokio::spawn(osint(self.snusbase.clone(), self.bulkvs.clone(), ctx, msg, args)); 
            }
            _ => { tokio::spawn(help(ctx, msg)); }
        }
    }

    async fn interaction_create(
        &self, 
        ctx: serenity::client::Context, 
        interaction: Interaction
    ) {
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
                "announce_osint" => {
                    let response = commands::announce_osint::run(
                            command.data.options(), 
                            &ctx, 
                            self.state.clone()
                        ).await.expect("Failed to run command!");
                
                    Some(response)
                },
                "development" => {
                    let response = commands::development::run(
                            command.data.options(), 
                            &ctx, 
                            self.ubisoft_api.clone()
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
    async fn ready(&self, ctx: serenity::client::Context, ready: Ready) {
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
                commands::announce_osint::register(),
                commands::development::register()
            ])
            .await.expect("Failed to register guild commands!");

        let command_names = commands
            .iter()
            .map(|x| x.name.clone())
            .collect::<Vec<String>>();
        println!("I now have the following guild slash commands: {command_names:?}");
    }
}
pub async fn help( 
    ctx: serenity::client::Context,
    msg: Message 
) {
    let _ = send_embed(
        &ctx, 
        &msg, 
        "All Sections - Help", 
        concat!("**R6 Economy Command List**:\n",
            "- `>>r6 econ analyze <item name | item id>`\n",
            "- `>>r6 econ graph <item name | item id>`\n",
            "- `>>r6 econ profit <purchased at> <item name | item id>`\n",
            "- `>>r6 econ list <(optional) page #>`\n",
            "- `>>r6 econ help`\n",
            "**R6 OPSEC Command List**:\n",
            "- `>>opsec <pc | xbox | psn> <username>`\n",
            "- `>>opsec namefind <username1, username2, ...>`\n",
            "- `>>opsec help`\n",
            "**OSINT Command List**:\n",
            "- `>>osint email <email>`\n",
            "- `>>osint username <username>`\n",
            "- `>>osint password <password>`\n",
            "- `>>osint name <name>`\n",
            "- `>>osint hash <hash>`\n",
            "- `>>osint dehash <hash>`\n",
            "- `>>osint rehash <password>`\n",
            "- `>>osint ip <ip>`\n",
            "- `>>osint last_ip <last ip>`\n",
            "- `>>osint phone <phone number>`\n",
            "**Ban Watch Command List**:\n",
            "- **Still under development, stay cozy...**\n",
            "**Admin Command List**:\n",
            "- `>>r6 admin whitelist <section> <user id>`\n",
            "- `>>r6 admin blacklist <section> <user id>`\n",
            "- `>>r6 admin help`\n",
            "\n\n*Developed by @hiibolt on GitHub*"),
            get_random_anime_girl()
    ).await
        .expect("Failed to send embed!");
}

#[tokio::main]
async fn main() -> Result<()> {
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

    // Build the BulkVS API
    let bulkvs = Arc::new(
        Mutex::new(
            BulkVS::new().expect("Could not create BulkVS API!")
        )
    );

    // Build the Ubisoft API and log in
    let ubisoft_api = Arc::new(
        Mutex::new(
            Ubisoft::new(
                env::var("UBISOFT_AUTH_EMAIL")
                    .expect("Could not find UBISOFT_AUTH_EMAIL in the environment!"),
                env::var("UBISOFT_AUTH_PW")
                    .expect("Could not find UBISOFT_AUTH_PW in the environment!")
            )
        )
    );

    // Build the Database object and log in
    let database = Arc::new(
        Mutex::new(
            Database::new(
                env::var("DATABASE_API_KEY")
                    .expect("Could not find DATABASE_API_KEY in the environment!")
            )
        )
    );

    // Test that the database is operational
    if let Err(e) = database
        .lock().await
        .verify_db() {
        println!("Failed to update DB with reason `{e}`!");
    }

    // Start login process
    tokio::spawn(Ubisoft::auto_login( ubisoft_api.clone()));

    // Start autosave
    tokio::spawn(helper::autosave( state.clone() ));

    // Start autopull
    tokio::spawn(helper::autopull( state.clone() ));

    // Build client with state
    let mut client =
        Client::builder(&token, intents)
        .event_handler(Bot {
            snusbase,
            bulkvs,
            ubisoft_api,
            state,
            database
        })
        .activity(ActivityData {
            name: String::from("serverspace"),
            kind: ActivityType::Competing,
            state: Some(String::from("Written and maintained by @hiibolt")),
            url: Some(Url::parse("https://github.com/hiibolt/").expect("Hardcoded URL is invalid!"))
        })
        .status(OnlineStatus::DoNotDisturb)
        .await.expect("Err creating client");
    
    // Start r6rs
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }

    Ok(())
}