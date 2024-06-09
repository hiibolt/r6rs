use std::sync::Arc;
use serenity::all::{CommandOptionType, Context, CreateCommandOption, CreateMessage, ResolvedValue};
use tokio::sync::Mutex;

use serenity::builder::CreateCommand;
use serenity::model::application::ResolvedOption;

use crate::State;

pub async fn run<'a>(
    options: Vec<ResolvedOption<'a>>,
    ctx:      &Context,
    state:    Arc<Mutex<State>>
) -> Result<String, serenity::Error>  {
    let id_list_values = state.lock().await
        .bot_data["whitelisted_user_ids"]["opsec"].clone();

    println!("Ids: {:?}", id_list_values);

    let id_list: Vec<serenity::model::id::UserId> = id_list_values
        .as_array().expect("The user id whitelists must be lists, even if it's 0-1 users!")
        .into_iter()
        .map(|val| serenity::model::id::UserId::new(val.as_u64().expect("At least one user id is not a number!")))
        .collect();
    
    if let Some(ResolvedOption {
        value: ResolvedValue::String(message),
        ..
    }) = options.get(0) {
        println!("Message: {:?}", message);
        
        let builder = CreateMessage::new().content(*message);

        for id in id_list {
            println!("Id: {:?}", id);
    
            id.create_dm_channel(ctx.clone())
                .await
                .expect("Failed to send message!")
                .id
                .send_message(ctx.clone(), builder.clone())
                .await.expect("Failed to send message!");
        }
    }

    

    Ok("Successfully sent announcement command!".to_string())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("announce_opsec")
        .description("Announces to all whitelisted users.")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "message", "What you'd like to say")
                .required(true),
        )
}