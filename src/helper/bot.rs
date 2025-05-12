use super::{
    command::R6RSCommand,
    lib::{ edit_embed, get_random_anime_girl, send_embed }
};
use crate::{
    apis::{BulkVS, Snusbase, Ubisoft}, 
    error, info, startup, warn
};

use std::{
    collections::{HashMap, VecDeque}, env, net::TcpStream, sync::{atomic::AtomicU16, Arc}, time::SystemTime
};

use tokio::sync::Mutex;
use serde_json::Value;
use serenity::{all::{ChannelId, CreateAttachment, CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, GuildId, Interaction, ResolvedValue, User}, async_trait};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use colored::Colorize;
use tungstenite::{stream::MaybeTlsStream, WebSocket};

pub enum Sendable {
    DiscordResponseSender(DiscordResponseSender),
    _WebAPIResponseSender(WebAPIResponseSender),
    Other
}
pub struct WebAPIResponseSender {
    pub _title: String,
    pub _body: String,
    pub _image: String,
    pub _socket_sender: WebSocket<MaybeTlsStream<TcpStream>>
}
#[derive(Clone)]
pub struct DiscordResponseSender {
    pub ctx: serenity::client::Context,
    pub title: String,
    pub body: String,
    pub image: String,
    pub channel_id: ChannelId,
    pub author: User,
    pub message: Option<Message>,
    pub start_time: SystemTime,
    pub ongoing_edits: Arc<AtomicU16>
}
impl Sendable {
    pub async fn send(
        &mut self,
        title: String,
        body: String,
        image: String
    ) -> Result<(), String> {
        match self {
            Sendable::DiscordResponseSender(sender) => {
                sender.title = title;
                sender.body = body;
                sender.image = image;

                sender.message = Some(send_embed(
                    &sender.ctx, 
                    &sender.channel_id, 
                    &sender.title, 
                    &(sender.body.clone() + "\n\n-# Still working..."), 
                    &sender.image
                ).await
                    .map_err(|e| format!("Failed to send message!\n\n{e:?}"))?);
            },
            _ => {
                panic!("Invalid sender type!");
            }
        }

        Ok(())
    }
    pub async fn send_premade_embed(
        &mut self,
        builder: CreateMessage
    ) -> Result<(), String> {
        match self {
            Sendable::DiscordResponseSender(sender) => {
                sender.channel_id.send_message(sender.ctx.clone(), builder).await
                    .map_err(|e| format!("An error occurred sending the embed!\n\n{e:?}"))?;
            },
            _ => {
                panic!("Invalid sender type!");
            }
        }

        Ok(())
    }
    pub async fn send_text_file(
        &mut self,
        content: String,
        builder: CreateMessage
    ) -> Result<(), String> {
        match self {
            Sendable::DiscordResponseSender(sender) => {
                sender.channel_id.send_files(
                    sender.ctx.clone(),
                    std::iter::once(CreateAttachment::bytes(
                        content.as_bytes(),
                        "full_dump.txt"
                    )),
                    builder
                ).await
                    .map_err(|e| format!("An error occurred sending the embed!\n\n{e:?}"))?;
            },
            _ => {
                panic!("Invalid sender type!");
            }
        }

        Ok(())
    }

    pub async fn add_line(
        &mut self,
        body: String
    ) -> Result<(), String> {
        match self {
            Sendable::DiscordResponseSender(sender) => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                sender.ongoing_edits.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                let mut tries = 10;
                while sender.message.is_none() {
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    tries -= 1;

                    if tries == 0 {
                        return Err(String::from("Message wasn't created even after a full second!"));
                    }
                }

                for cycle in 1..=10 {
                    if let Ok(msg) = sender.ctx.http.get_message(
                        sender.channel_id, 
                        sender.message.clone().unwrap().id
                    ).await {
                        let actual_body = msg
                            .embeds
                            .into_iter()
                            .flat_map(|embed| embed.description)
                            .collect::<Vec<String>>()
                            .join("");

                        let edit_number = sender.ongoing_edits.load(std::sync::atomic::Ordering::SeqCst);
                        if actual_body == sender.body.clone() + "\n\n-# Still working..." {
                            info!("Safe to complete edit #{edit_number}!");

                            break;
                        }
                    }

                    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

                    info!("Waiting for edit to complete, {cycle} cycles left...");

                    if cycle == 10 {
                        warn!("Failed to edit message after 10 tries! Force editing.");
                    }
                }

                sender.body += &body;
                edit_embed(
                    &sender.ctx, 
                    &mut sender.message.clone().expect("Unreachable!"), 
                    &sender.title, 
                    &(sender.body.clone() + "\n\n-# Still working..."), 
                    &sender.image
                ).await;
                sender.ongoing_edits.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
            },
            _ => {
                panic!("Invalid sender type!");
            }
        }

        Ok(())
    }

    pub async fn finalize(
        &mut self
    ) -> Result<(), String> {
        match self {
            Sendable::DiscordResponseSender(sender) => {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                // Wait for any previous edits
                for cycle in 1..=10 {
                    if sender.ongoing_edits.load(std::sync::atomic::Ordering::SeqCst) == 0 {
                        info!("Safe to finalize!");

                        break;
                    }

                    tokio::time::sleep(std::time::Duration::from_millis(5000)).await;

                    info!("Waiting for all edits to complete, {cycle} cycles left...");

                    if cycle == 10 {
                        warn!("Failed to edit message after 10 tries! Force editing.");
                    }
                }

                // Add completion notification to body
                sender.body += "\n\n-# Command completed";

                if let Ok(ms_to_complete) = sender.start_time.elapsed().and_then(|time| Ok(time.as_millis())) {
                    sender.body += &format!(" in {ms_to_complete}ms");
                }

                // Edit the message
                edit_embed(
                    &sender.ctx, 
                    &mut sender.message.clone().ok_or(String::from("No message to edit!"))?, 
                    &sender.title, 
                    &sender.body, 
                    &sender.image
                ).await;
            },
            _ => {
                panic!("Invalid sender type!");
            }
        }

        Ok(())
    }
}


#[derive(Clone)]
pub struct BackendHandles {
    pub ubisoft_api: Arc<Mutex<Ubisoft>>,
    pub snusbase:    Arc<Mutex<Snusbase>>,
    pub bulkvs:      Arc<Mutex<BulkVS>>,
    pub state:       Arc<Mutex<State>>
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

        // Double check that the message is a command meant for the bot
        if let Ok(val) = std::env::var("DEV_MODE") {
            if val == "true" {
                if args[0].chars().take(3).collect::<String>() != "dev" {
                    if &args[0].chars().take(2).collect::<String>() == ">>" {
                        warn!("Got standard command, but you're in dev mode!");
                    }
                    return;
                }
                args[0] = args[0].chars().skip(3).collect();
            }
        }       
        if &args[0].chars().take(2).collect::<String>() != ">>" {
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
                    &format!("Failed for reason:\n\nCould not download your file! Was it too big?"), 
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
                        &format!("Failed for reason:\n\nFailed to convert your file to a UTF-8 string! This bot only supports *text* files as arguments :)"), 
                        get_random_anime_girl()
                    ).await.unwrap();
                    return;
                }
            };

            args.push_back(st);
        }

        // Call the command
        let content = &msg.content;
        info!("Received command: {content}");
        let sendable = Arc::new(Mutex::new(Sendable::DiscordResponseSender(DiscordResponseSender {
            ctx: ctx.clone(),
            title: String::new(),
            body: String::new(),
            image: String::new(),
            channel_id: msg.channel_id,
            author: msg.author.clone(),
            message: None,
            start_time: SystemTime::now(),
            ongoing_edits: Arc::new(AtomicU16::new(0))
        })));
        if let Err(err) = self.root_command.lock().await.call(
            self.backend_handles.clone(),
            sendable.clone(),
            args
        ).await {
            error!("Failed! [{err}]");
            sendable.lock().await.send(
                "R6RS - Error".to_string(),
                format!("Failed for reason:\n\n{err}").to_string(),
                get_random_anime_girl().to_string()
            ).await
                .expect("Failed to send message!");
            sendable.lock().await
                .finalize()
                .await.expect("Failed to finalize message!");
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
            let sendable = Sendable::DiscordResponseSender(DiscordResponseSender {
                ctx: ctx.clone(),
                title: String::new(),
                body: String::new(),
                image: String::new(),
                channel_id: command.channel_id,
                author: command.member.clone().unwrap().user.clone(),
                message: None,
                start_time: SystemTime::now(),
                ongoing_edits: Arc::new(AtomicU16::new(0))
            });

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
                Arc::new(Mutex::new(sendable)),
                args
            ).await {
                error!("Failed! [{err}]");
                send_embed(
                    &ctx, 
                    &command.channel_id, 
                    "R6RS - Error", 
                    &format!("Failed for reason:\n\n{err}"),
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