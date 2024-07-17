mod helper;
mod sections;
mod apis;

use crate::{
    helper::{ send_embed, get_random_anime_girl },
    apis::{ Snusbase, BulkVS, Ubisoft, Database }
};
use std::{
    collections::VecDeque, env, fs::read_to_string, sync::Arc
};

use apis::database::CommandEntry;
use helper::{inject_documentation, BackendHandles, GenericMessage, R6RSCommand};
use tokio::sync::Mutex;
use serde_json::Value;
use serenity::{all::{ActivityData, ActivityType, GuildId, Interaction, OnlineStatus, ResolvedValue}, async_trait};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use url::Url;
use std::collections::HashMap;
use anyhow::Result;
use colored::Colorize;

struct State {
    bot_data: Value,
    id_list: HashMap<String, String>,
    market_data: Value
}

struct Bot {
    root_command: Arc<Mutex<R6RSCommand>>,

    backend_handles: BackendHandles
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(
        &self, 
        ctx: serenity::client::Context, 
        msg: Message
    ) {
        let args: VecDeque<String> = msg.content
            .clone()
            .split(' ')
            .map(|i| String::from(i))
            .collect();
        let user_id: u64 = msg.author.id.get();
        let message_id: u64 = msg.id.get();
        let server_id = msg.guild_id
            .and_then(|gid| Some(gid.get()))
            .unwrap_or(0u64);

        let front_arg = args.clone().pop_front().unwrap();

        if &front_arg.chars().take(2).collect::<String>() != ">>" {
            return;
        }

        if let Err(e) = self.backend_handles.database
            .lock().await
            .upload_command(CommandEntry { 
                message_id,
                user_id,
                server_id,
                command: msg.content.clone()
            }) {
            warn!("Failed to update DB with reason `{e}`!");
        }

        let content = &msg.content;
        info!("Received command: {content}");
        
        if let Err(err) = self.root_command.lock().await.call(
            self.backend_handles.clone(),
            ctx.clone(), 
            GenericMessage {
                channel_id: msg.channel_id,
                content: msg.content.clone(),
                author: msg.author.clone(),
            }, 
            args
        ).await {
            error!("Failed! [{err}]");
            send_embed(
                &ctx, 
                &msg.channel_id, 
                "R6RS - Error", 
                &format!("Failed for reason:\n\n\"{err}\""), 
                get_random_anime_girl()
            ).await.unwrap();
        }
    }

    async fn interaction_create(
        &self, 
        ctx: serenity::client::Context, 
        interaction: Interaction
    ) {
        if let Interaction::Command(command) = interaction {
            let command_name = &command.data.name;

            // Convert the slash command back into a standard command
            let mut args: VecDeque<String> = command_name
                .split('-')
                .enumerate()
                .map(|(ind, st)| {
                    if ind != 0 {
                        return st.to_string();
                    }
                    format!(">>{st}")
                })
                .collect();
            let mut options: VecDeque<String> = command.data.options()
                .iter()
                .map(|opt| {
                    if let ResolvedValue::String(st) = opt.value {
                        return st.to_owned();
                    }
                    panic!("Somehow recieved an option that wasn't a string!");
                })
                .collect();
            args.append(&mut options);

            // Logging
            info!("Received command interaction: {command_name} with args {args:?}");

            // Build the message
            let message = GenericMessage {
                channel_id: command.channel_id,
                content: command.data.name.clone(),
                author: command.member.clone().unwrap().user.clone()
            };

            // Log the slash command to the database
            let user_id: u64 = command.member.clone().unwrap().user.id.get();
            let message_id: u64 = command.id.get();
            let server_id = command.guild_id
                .and_then(|gid| Some(gid.get()))
                .unwrap_or(0u64);
            if let Err(e) = self.backend_handles.database
                .lock().await
                .upload_command(CommandEntry { 
                    message_id,
                    user_id,
                    server_id,
                    command: message.content.clone() + " - [slash command]"
                }) {
                warn!("Failed to update DB with reason `{e}`!");
            }

            // Call the command
            if let Err(err) = self.root_command.lock().await.call(
                self.backend_handles.clone(),
                ctx.clone(), 
                message, 
                args
            ).await {
                error!("Failed! [{err}]");
                send_embed(
                    &ctx, 
                    &command.channel_id, 
                    "R6RS - Error", 
                    &format!("Failed for reason:\n\n\"{err}\""),
                    get_random_anime_girl()
                ).await.unwrap();
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, ctx: serenity::client::Context, ready: Ready) {
        let guild_ids: Vec<GuildId> = env::var("GUILD_ID")
            .expect("Expected GUILD_ID in environment")
            .split(',')
            .map(|st| GuildId::new(st.parse().expect("GUILD_ID must be an integer")))
            .collect();
        startup!("Preparing to inject commands into the following guilds: {guild_ids:#?}");

        let auto_generated_commands = self.root_command
            .lock().await
            .build_commands("".into())
            .await;

        for guild_id in &guild_ids {
            let commands = match guild_id
                .set_commands(&ctx.http, auto_generated_commands.clone())
                .await {
                    Ok(commands) => commands,
                    Err(why) => {
                        for x in 0..auto_generated_commands.len() {
                            let copy = &auto_generated_commands[x];
                            println!("Command {x}:\n{copy:#?}");
                        }

                        error!("Failed to register commands: {why:#?}");
                        return;
                    }
            };

            let command_names = commands
                .iter()
                .map(|x| x.name.clone())
                .collect::<Vec<String>>();

            startup!("In server {guild_id:?}, I now have the following guild slash commands: {command_names:#?}");
            
            // Wait a second to avoid rate limiting
            if guild_ids.len() > 1 {
                startup!("Waiting for 1 second to avoid rate limiting...");
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                startup!("Done!");
            }
        }

        let bot_name = ready.user.name.clone();
        startup!("Bot \"{bot_name}\" is connected with data!");
    }
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
        warn!("Failed to update DB with reason `{e}`!");
    }

    // Start login process
    tokio::spawn(Ubisoft::auto_login( ubisoft_api.clone()));

    // Start autosave
    tokio::spawn(helper::autosave( state.clone() ));

    // Start autopull
    tokio::spawn(helper::autopull( state.clone() ));

    let admin_commands   = sections::admin::build_admin_commands().await;
    let econ_commands    = sections::econ::build_econ_commands().await;
    let osint_commands   = sections::osint::build_osint_commands().await;
    let opsec_commands   = sections::opsec::build_opsec_commands().await;
    let mut root_command = R6RSCommand::new_root(
        String::from("R6RS is a general purpose bot, orignally intended for Rainbow Six Siege, but since multipurposed into a powerful general OSINT tool."),
        String::from("Commands")
    );
    let mut r6_root_command = R6RSCommand::new_root(
        String::from("Commands specifically related to R6."),
        String::from("R6")
    );
    r6_root_command.attach(
        String::from("econ"),
        econ_commands
    );
    r6_root_command.attach(
        String::from("opsec"),
        opsec_commands
    );
    root_command.attach(
        String::from(">>r6"),
        r6_root_command
    );
    root_command.attach(
        String::from(">>admin"),
        admin_commands
    );
    root_command.attach(
        String::from(">>osint"),
        osint_commands
    );

    // Write command documentation
    inject_documentation(
        &root_command.print_help(String::from(""), 2, true).await
    ).await?;

    let backend_handles = BackendHandles {
        ubisoft_api,
        snusbase,
        bulkvs,
        database,
        state
    };

    // Build client with state
    let mut client =
        Client::builder(&token, intents)
        .event_handler(Bot {
            root_command: Arc::new(Mutex::new(root_command)),

            backend_handles
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
    client.start().await?;

    Ok(())
}