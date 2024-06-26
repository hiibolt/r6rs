use crate::Message;
use crate::State;
use crate::{ Arc, Mutex };
use crate::read_to_string;
use std::fs::OpenOptions;
use std::io::Write;
use tokio::time::{ sleep, Duration };
use serenity::model::colour::Colour;
use serenity::all::EditMessage;
use serenity::all::{ CreateEmbed, CreateMessage };
use rand::prelude::SliceRandom;

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

    println!("[Succesfully saved! :3]");
}
pub async fn autosave( state: Arc<Mutex<State>> ) {
    loop {
        save( state.clone() ).await;

        sleep(Duration::from_secs(120)).await;
    }
}
pub async fn autopull( state: Arc<Mutex<State>> ) {
    loop {
        println!("[Pulled market data :3]");

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
    msg: &Message,
    title: &str,
    description: &str,
    url: &str
) -> Result<Message, String> {
    let embed = CreateEmbed::new()
        .title(title)
        .description(description)
        .color(get_random_color())
        .thumbnail(url);
    
    let builder = CreateMessage::new().embed(embed);

    msg.channel_id.send_message(&ctx.http, builder).await.map_err(|e| format!("{e:?}"))
        .map_err(|_| String::from("Failed to send error!s"))
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
        &msg, 
        "Not yet implemented!", 
        &format!("The command **{cmd}** exists but is not yet implemented! While I work, stay cozy :3"), 
        get_random_anime_girl()
    ).await
        .unwrap();
}
pub async fn no_access(
    ctx: serenity::client::Context,
    msg: Message,
    cmd: &str,
    id: u64
) {
    send_embed(
        &ctx, 
        &msg, 
        "You don't have access to this command!", 
        &format!("You (**@{id}**) aren't authorized to use **{cmd}**.\n\n*Contact @hiibolt to purchase access or if this is in error.*"), 
        get_random_anime_girl()
    ).await
        .unwrap();
}
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