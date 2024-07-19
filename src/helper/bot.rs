use super::{
    command::R6RSCommand,
    lib::{ get_random_anime_girl, send_embed }
};
use crate::{
    apis::{BulkVS, Database, Snusbase, Ubisoft, database::CommandEntry}, 
    error, info, startup, warn
};

use std::{
    collections::{VecDeque, HashMap}, 
    env,
    sync::Arc
};

use tokio::sync::Mutex;
use serde_json::Value;
use serenity::{all::{ChannelId, CreateInteractionResponse, CreateInteractionResponseMessage, GuildId, Interaction, ResolvedValue, User}, async_trait};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use colored::Colorize;


#[derive(Clone)]
pub struct GenericMessage {
    pub channel_id: ChannelId,
    pub content: String,
    pub author: User
}
#[derive(Clone)]
pub struct BackendHandles {
    pub ubisoft_api: Arc<Mutex<Ubisoft>>,
    pub snusbase:    Arc<Mutex<Snusbase>>,
    pub bulkvs:      Arc<Mutex<BulkVS>>,
    pub state:       Arc<Mutex<State>>,
    pub database:    Arc<Mutex<Database>>
}
pub struct State {
    pub bot_data: Value,
    pub id_list: HashMap<String, String>,
    pub market_data: Value
}
pub struct Bot {
    pub root_command: Arc<Mutex<R6RSCommand>>,

    pub backend_handles: BackendHandles
}

#[async_trait]
impl EventHandler for Bot {
    async fn message(
        &self, 
        ctx: serenity::client::Context, 
        msg: Message
    ) {
        // Extract the command, user, and server IDs
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

        // Double check that the message is a command meant for the bot
        let front_arg = args.clone().pop_front().unwrap();
        if &front_arg.chars().take(2).collect::<String>() != ">>" {
            return;
        }

        // Convert any attachments to strings and add them to the args
        for attachment in msg.attachments {
            // Download the attachment
            let mut bytes = if let Ok(bytes) = attachment.download().await {
                bytes
            } else {
                error!("Failed to download attachment!");
                send_embed(
                    &ctx, 
                    &msg.channel_id, 
                    "R6RS - Error", 
                    &format!("Failed for reason:\n\n\"Could not download your file! Was it too big?\""), 
                    get_random_anime_girl()
                ).await.unwrap();
                return;
            };

            // Purge any invalid UTF-8 characters
            bytes = bytes
                .iter()
                .filter(|byte| byte.is_ascii())
                .map(|byte| byte.clone())
                .collect::<Vec<u8>>();

            let st = match String::from_utf8(bytes) {
                Ok(st) => st,
                Err(err) => {
                    error!("Failed to convert bytes into string! {err:#?}");
                    send_embed(
                        &ctx, 
                        &msg.channel_id, 
                        "R6RS - Error", 
                        &format!("Failed for reason:\n\n\"Failed to convert your file to a UTF-8 string! This bot only supports *text* files as arguments :)\""), 
                        get_random_anime_girl()
                    ).await.unwrap();
                    return;
                }
            };

            args.push_back(st);
        }

        // Log the command to the database
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

        // Call the command
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
            let mut options: VecDeque<String> = VecDeque::new();
            
            for opt in command.data.options() {
                if let ResolvedValue::String(st) = opt.value {
                    options.push_back(st.to_owned());

                    continue;
                }

                if let ResolvedValue::Attachment(att) = opt.value {
                    let bytes = if let Ok(bytes) = att.download().await {
                        bytes
                    } else {
                        // Let the user know you're about to error out
                        if let Err(why) = command.create_response(
                            &ctx.http, 
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new().content("` Failed to download your file! Was it too big? `")
                            )
                        ).await {
                            panic!("Cannot respond to slash command: {why}");
                        }
                        panic!("Failed to convert bytes into string!");
                    };

                    let st = if let Ok(st) = String::from_utf8(bytes) {
                        st
                    } else {
                        // Let the user know you're about to error out
                        if let Err(why) = command.create_response(
                            &ctx.http, 
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new().content("` Failed to convert your file into a string! `")
                            )
                        ).await {
                            panic!("Cannot respond to slash command: {why}");
                        }
                        panic!("Failed to convert bytes into string!");
                    };
                    
                    options.push_back(st);

                    continue;
                }

                panic!("Somehow recieved an option that wasn't a string!");
            }

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

            // Let the user know you're about to start working
            if let Err(why) = command.create_response(
                &ctx.http, 
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new().content("` Getting to work... `")
                )
            ).await {
                error!("Cannot respond to slash command: {why}");
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
            startup!("Preparing to inject commands into the following guild: {guild_id:?}");
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