mod helper;
mod commands;
mod sections;
mod apis;

use crate::{
    helper::{ send_embed, get_random_anime_girl },
    apis::{ Snusbase, BulkVS, Ubisoft, Database }
};
use std::{
    env,
    collections::VecDeque,
    fs::read_to_string,
    sync::Arc
};

use apis::database::CommandEntry;
use helper::{inject_documentation, startup, BackendHandles, R6RSCommand};
use tokio::sync::Mutex;
use serde_json::Value;
use serenity::{all::{ActivityData, ActivityType, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId, Interaction, OnlineStatus}, async_trait};
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
            msg.clone(), 
            args
        ).await {
            error!("Failed! [{err}]");
            send_embed(
                &ctx, 
                &msg, 
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
            info!("Received command interaction: {command_name}");

            let content = match command.data.name.as_str() {
                "announce" => {
                    let response = commands::announce_all::run(
                            command.data.options(), 
                            &ctx, 
                            self.backend_handles.state.clone()
                        ).await.expect("Failed to run command!");
                
                    Some(response)
                },
                "announce_opsec" => {
                    let response = commands::announce_opsec::run(
                            command.data.options(), 
                            &ctx, 
                            self.backend_handles.state.clone()
                        ).await.expect("Failed to run command!");
                
                    Some(response)
                },
                "announce_econ" => {
                    let response = commands::announce_econ::run(
                            command.data.options(), 
                            &ctx, 
                            self.backend_handles.state.clone()
                        ).await.expect("Failed to run command!");
                
                    Some(response)
                },
                "announce_osint" => {
                    let response = commands::announce_osint::run(
                            command.data.options(), 
                            &ctx, 
                            self.backend_handles.state.clone()
                        ).await.expect("Failed to run command!");
                
                    Some(response)
                },
                "development" => {
                    let response = commands::development::run(
                            command.data.options(), 
                            &ctx, 
                            self.backend_handles.ubisoft_api.clone()
                        ).await.expect("Failed to run command!");
                
                    Some(response)
                },
                _ => Some("not implemented :(".to_string()),
            };

            if let Some(content) = content {
                let data = CreateInteractionResponseMessage::new().content(content);
                let builder = CreateInteractionResponse::Message(data);
                if let Err(why) = command.create_response(&ctx.http, builder).await {
                    error!("Cannot respond to slash command: {why}");
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
        startup(&format!("I now have the following guild slash commands: {command_names:#?}"));

        startup(&format!("Bot \"{}\" is connected with data!", ready.user.name));
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
            "- `>>r6 econ transfer <(optional) email> <(optional) password>`\n",
            "- `>>r6 econ help`\n",
            "**R6 OPSEC Command List**:\n",
            "- `>>opsec <pc | xbox | psn> <username>`\n",
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
            "- `>>osint sherlock <username>`\n",
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