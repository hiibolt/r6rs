use std::{
    env,
    collections::{ VecDeque }
};

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

async fn econ( ctx: Context, msg: Message, args: VecDeque<String> ) {
    todo!();
}
async fn stats( ctx: Context, msg: Message, args: VecDeque<String> ) {
    todo!();
}
async fn opsec( ctx: Context, msg: Message, args: VecDeque<String> ) {
    todo!();
}
async fn bans( ctx: Context, msg: Message, args: VecDeque<String> ) {
    todo!();
}
async fn admin( ctx: Context, msg: Message, args: VecDeque<String> ) {
    todo!();
}
async fn help( ctx: Context, msg: Message, args: VecDeque<String> ) {
    todo!();
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let mut args: VecDeque<String> = msg.content
            .clone()
            .split(' ')
            .map(|i| String::from(i))
            .collect();

        if args.pop_front() != Some(String::from("r6")) {
            return;
        }


        match args
            .pop_front()
            .unwrap_or(String::from("help"))
            .as_str()
        {
            "econ" => { tokio::spawn(econ(ctx, msg, args)); },
            "stats" => { tokio::spawn(stats(ctx, msg, args)); },
            "opsec" => { tokio::spawn(opsec(ctx, msg, args)); },
            "bans" => { tokio::spawn(opsec(ctx, msg, args)); },
            "admin" => { tokio::spawn(admin(ctx, msg, args)); },
            _ => { tokio::spawn(help(ctx, msg, args)); }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_BOT_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client =
        Client::builder(&token, intents).event_handler(Handler).await.expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}