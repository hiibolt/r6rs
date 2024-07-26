use super::bot::{BackendHandles, Sendable};
use crate::{
    error, info, startup, daemon,
    Message, State,
    read_to_string,
    Arc, Mutex
};

use std::{
    fs::OpenOptions,
    io::Write,
    collections::VecDeque,
};

use tokio::time::{sleep, Duration};
use futures::future::{Future, BoxFuture};
use anyhow::{Result, Context};
use colored::Colorize;
use rand::prelude::SliceRandom;
use serenity::{
    all::{ChannelId, CreateEmbed, CreateMessage, EditMessage},
    model::colour::Colour,
};

pub struct AsyncFnPtr<R> {
    func: Box<dyn Fn(
        BackendHandles,
        Arc<Mutex<Sendable>>,
        VecDeque<String>
    ) -> BoxFuture<'static, R> + Send + Sync + 'static>
}
impl <R> AsyncFnPtr<R> {
    pub fn new<F>(
        f: fn(
            BackendHandles,
            Arc<Mutex<Sendable>>,
            VecDeque<String>
        ) -> F
    ) -> AsyncFnPtr<F::Output> 
    where 
        F: Future<Output = R> + Send + Sync + 'static
    {
        AsyncFnPtr {
            func: Box::new(move |backend_handles, sendable, args| Box::pin(f(backend_handles, sendable, args))),
        }
    }
    pub async fn run(
        &self,
        backend_handles: BackendHandles,
        sendable: Arc<Mutex<Sendable>>,
        args: VecDeque<String>
    ) -> R { 
        (self.func)(backend_handles, sendable, args).await
    }
}


pub async fn inject_documentation(
    body: &str
) -> Result<()> {
    // Load the documentation template from `assets/README_TEMPLATE.md`
    let template = read_to_string("assets/README_TEMPLATE.md")
        .context("Failed to read `assets/README_TEMPLATE.md`! Does the file exist?")?;

    // Inject the body into the template
    let inject_marker = "<!-- INJECT MARKER -->";
    let injected = template.replace(inject_marker, body);

    if injected != read_to_string("README.md")
        .context("Failed to read `README.md`! Does the file exist?")? {
        // Write the injected template to `README.md`
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open("README.md")
            .context("Failed to open file handle to `README.md`! Does the file exist?")?
            .write_all(injected.as_bytes())
            .context("Failed to write to `README.md`! Is the file in use?")?;
    } else {
        startup!("Documentation is already up to date! :3");
    }


    startup!("Succesfully injected documentation! :3");

    Ok(())
}
pub async fn save( state: Arc<Mutex<State>> ) {
    let bot_data_serialized = &state
        .lock().await
        .bot_data
        .to_string();

    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("assets/bot_data.json")
        .expect("Failed to open file handle to `assets/bot_data.json`! Does the file exist?")
        .write_all(bot_data_serialized.as_bytes())
        .expect("Failed to write to `assets/bot_data.json`! Is the file in use?");

    daemon!("Succesfully saved! :3");
}
pub async fn autosave( state: Arc<Mutex<State>> ) {
    loop {
        save( state.clone() ).await;

        sleep(Duration::from_secs(120)).await;
    }
}
pub async fn autopull( state: Arc<Mutex<State>> ) {
    loop {
        daemon!("Pulled market data :3");

        let market_data_contents: String = read_to_string("assets/data.json")
            .expect("Could not find 'assets/data.json', please ensure you have created one!");
    
        state.lock()
            .await
            .market_data = serde_json::from_str(&market_data_contents)
                .expect("Could not parse the contents of 'data.json'!");

        sleep(Duration::from_secs(60)).await;
    }
}
pub async fn send_embed(
    ctx: &serenity::client::Context,
    channel_id: &ChannelId,
    title: &str,
    description: &str,
    url: &str
) -> Result<Message, String> {
    //println!("Title: {title}\nDescription: {description}\nURL: {url}");

    let embed = CreateEmbed::new()
        .title(title)
        .description(description)
        .color(get_random_color())
        .thumbnail(url);
    
    let builder = CreateMessage::new().embed(embed);

    channel_id.send_message(&ctx.http, builder).await.map_err(|e| format!("{e:?}"))
        .map_err(|e| format!("Failed to send embed!\n\n{e:?}"))
}
pub async fn edit_embed(
    ctx: &serenity::client::Context,
    msg: &mut Message,
    title: &str,
    description: &str,
    url: &str
) {
    let embed_builder = CreateEmbed::new()
        .title(title)
        .description(description)
        .color(get_random_color())
        .thumbnail(url);
    let edit_builder = EditMessage::new().embed(embed_builder);

    msg.edit(ctx, edit_builder).await.unwrap();
}
pub async fn dm_to_person (
    ctx: serenity::all::Context,
    user_id: serenity::model::id::UserId,
    message: String
) -> Result<(), serenity::Error> {
    let builder: CreateMessage = CreateMessage::new().content(message);

    if let Ok(private_channel) = user_id.create_dm_channel(ctx.clone())
        .await {
        let channel_id = &private_channel.id;
        info!("Channel Id: {channel_id:?}");

        if let Err(e) = private_channel
            .id
            .send_message(ctx, builder.clone())
            .await 
        {
            error!("Error sending message to user: {e:?}");
        }
    }

    Ok(())
}
pub fn get_random_color () -> Colour {
    vec!(
        Colour::FABLED_PINK,
        Colour::MEIBE_PINK,
        Colour::DARK_MAGENTA,
        Colour::MAGENTA
    ).choose(&mut rand::thread_rng()).unwrap().clone()
}
pub async fn _unimplemented(
    ctx: serenity::client::Context,
    msg: Message,
    cmd: &str
) {
    send_embed(
        &ctx, 
        &msg.channel_id, 
        "Not yet implemented!", 
        &format!("The command **{cmd}** exists but is not yet implemented! While I work, stay cozy :3"), 
        get_random_anime_girl()
    ).await
        .unwrap();
}
/*
pub async fn no_access(
    ctx: serenity::client::Context,
    msg: GenericMessage,
    cmd: &str,
    id: u64
) {
    send_embed(
        &ctx, 
        &msg.channel_id, 
        "You don't have access to this command!", 
        &format!("You (**@{id}**) aren't authorized to use **{cmd}**.\n\n*Contact @hiibolt to purchase access or if this is in error.*"), 
        get_random_anime_girl()
    ).await
        .unwrap();
} */
pub fn get_random_anime_girl() -> &'static str {
    [
        "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4",
        "https://github.com/hiibolt/hiibolt/assets/91273156/831e2922-cdcb-409d-a919-1a72fbe56ff4",
        "https://github.com/hiibolt/hiibolt/assets/91273156/9098eb3f-d883-4a8b-8c6b-525869eac2a2",
        "https://github.com/hiibolt/hiibolt/assets/91273156/d8891401-df14-435b-89a5-c23da4c38354",
        "https://github.com/hiibolt/hiibolt/assets/91273156/353dea2e-f436-4289-9a10-37f9a23e3ee6",
        "https://github.com/hiibolt/hiibolt/assets/91273156/b3cf1ffd-874b-403c-9716-dce4d4f03ae0"
    ].choose(&mut rand::thread_rng()).expect("Unreachable!")
}