use crate::Message;
use crate::Context;
use crate::State;
use crate::{ Arc, Mutex };
use crate::read_to_string;
use std::fs::OpenOptions;
use std::io::Write;
use tokio::time::{ sleep, Duration };
use serenity::all::EditMessage;
use serenity::all::{ CreateEmbed, CreateMessage };

pub async fn save( state: Arc<Mutex<State>> ) {
    let bot_data_serialized = &state
        .lock().await
        .bot_data
        .to_string();
    let id_list_serialized = serde_json::to_string(&state
        .lock().await
        .id_list)
        .expect("Failed to serialize ID list! Potentially unreachable?");
    let market_data_serialized = &state
        .lock().await
        .market_data
        .to_string();

    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("assets/bot_data.json")
        .expect("Failed to open file handle to `assets/bot_data.json`! Does the file exist?")
        .write_all(bot_data_serialized.as_bytes())
        .expect("Failed to write to `assets/bot_data.json`! Is the file in use?");
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("assets/id_list.json")
        .expect("Failed to open file handle to `assets/id_list.json`! Does the file exist?")
        .write_all(id_list_serialized.as_bytes())
        .expect("Failed to write to `assets/id_list.json`! Is the file in use?");
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("assets/market_data.json")
        .expect("Failed to open file handle to `assets/market_data.json`! Does the file exist?")
        .write_all(market_data_serialized.as_bytes())
        .expect("Failed to write to `assets/market_data.json`! Is the file in use?");
    
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

        let market_data_contents: String = read_to_string("assets/market_data.json")
            .expect("Could not find 'assets/market_data.json', please ensure you have created one!");
    
        state.lock()
            .await
            .market_data = serde_json::from_str(&market_data_contents)
                .expect("Could not parse the contents of 'market_data.json'!");

        sleep(Duration::from_secs(60)).await;
    }
}
pub async fn send_embed( ctx: &Context, msg: &Message, title: &str, description: &str, url: &str ) -> Result<Message, String> {
    let embed = CreateEmbed::new()
        .title(title)
        .description(description)
        .thumbnail(url);
    
    let builder = CreateMessage::new().embed(embed);

    msg.channel_id.send_message(&ctx.http, builder).await.map_err(|e| format!("{e:?}"))
        .map_err(|_| String::from("Failed to send error!s"))
}
pub async fn edit_embed( ctx: &Context, msg: &mut Message, title: &str, description: &str, url: &str ) {
    let embed_builder = CreateEmbed::new()
        .title(title)
        .description(description)
        .thumbnail(url);
    let edit_builder = EditMessage::new().embed(embed_builder);

    msg.edit(ctx, edit_builder).await.unwrap();
}
pub async fn unimplemented( ctx: Context, msg: Message, cmd: &str ) {
    send_embed(
        &ctx, 
        &msg, 
        "Not yet implemented!", 
        &format!("The command **{cmd}** exists but is not yet implemented! While I work, stay cozy :3"), 
        "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
    ).await
        .unwrap();
}
pub async fn no_access( ctx: Context, msg: Message, cmd: &str, id: u64 ) {
    send_embed(
        &ctx, 
        &msg, 
        "You don't have access to this command!", 
        &format!("You (**@{id}**) aren't authorized to use **{cmd}**.\n\n*Contact @hiibolt to purchase access or if this is in error.*"), 
        "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
    ).await
        .unwrap();
}