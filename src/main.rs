mod opsec;
mod admin;
mod bans;
mod econ;
mod help;
mod lib;

use crate::{
    econ::econ,
    opsec::opsec,
    bans::bans,
    admin::admin,
    help::help
};

use std::{
    env,
    collections::{ VecDeque },
    fs::read_to_string
};

use lib::{ no_access, unimplemented, send_embed };

use tokio::sync::Mutex;
use serde_json::{Value};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

#[derive(Debug)]
struct State {
    bot_data: Value,
    id_list: Value,
    market_data: Value
}

#[derive(Debug)]
struct Bot {
    state: Mutex<State>
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
                tokio::spawn(econ(ctx, msg, args));
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
                    no_access( ctx, msg.clone(), "econ", user_id ).await;
                    return;
                }

                // Otherwise, go ahead
                tokio::spawn(opsec(ctx, msg, args)); 
            },
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
                tokio::spawn(admin(ctx, msg, args));
            },
            _ => { tokio::spawn(help(ctx, msg, args)); }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected with data!", ready.user.name);
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
    let id_list_contents: String = read_to_string("assets/id_list.json")
        .expect("Could not find 'assets/id_list.json', please ensure you have created one!");
    let market_data_contents: String = read_to_string("assets/market_data.json")
        .expect("Could not find 'assets/market_data.json', please ensure you have created one!");
    
    // Build the state into an async mutex
    let state = Mutex::new(
        State {
            bot_data: serde_json::from_str(&bot_data_contents)
                .expect("Could not parse the contents of 'bot_data.json'!"),
            id_list: serde_json::from_str(&id_list_contents)
                .expect("Could not parse the contents of 'id_list.json'!"),
            market_data: serde_json::from_str(&market_data_contents)
                .expect("Could not parse the contents of 'market_data.json'!"),
        }
    );

    // Build client with state
    let mut client =
        Client::builder(&token, intents)
        .event_handler(Bot {
            state
        })
        .await.expect("Err creating client");
    
    // Start r6rs
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}