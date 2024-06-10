use std::sync::Arc;
use serde_json::Value;
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
    let econ_id_list_value = state.lock().await
        .bot_data["whitelisted_user_ids"]["econ"].clone();
    let opsec_id_list_value = state.lock().await
        .bot_data["whitelisted_user_ids"]["opsec"].clone();
    let econ_id_list_vec = econ_id_list_value
        .as_array().expect("The user id whitelists must be lists, even if it's 0-1 users!");
    let opsec_id_list_vec = opsec_id_list_value
        .as_array().expect("The user id whitelists must be lists, even if it's 0-1 users!");

    let id_list_values = econ_id_list_vec
        .into_iter()
        .chain(opsec_id_list_vec.into_iter())
        .cloned()
        .collect::<Vec<Value>>();

    println!("Ids: {:?}", id_list_values);

    let id_list: Vec<serenity::model::id::UserId> = vec!(Value::Number(525429484997050379u64.into()))
        .into_iter()
        .map(|val| serenity::model::id::UserId::new(val.as_u64().expect("At least one user id is not a number!")))
        .collect();
    
    if let Some(ResolvedOption {
        value: ResolvedValue::String(message),
        ..
    }) = options.get(0) {
        println!("Message: {:?}", message);
        
        for id in id_list.iter() {
            println!("Id: {:?}", id);

            let builder: CreateMessage = CreateMessage::new().content(*message);
    
            if let Ok(private_channel) = id.create_dm_channel(ctx.clone())
                .await {
                println!("Channel Id: {:?}", private_channel.id);

                if let Err(e) = private_channel
                    .id
                    .send_message(ctx.clone(), builder.clone())
                    .await {
                    println!("Error sending message to user: {:?}", e);
                }
            }
        }
    }

    

    Ok("Successfully sent announcement command!".to_string())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("announce")
        .description("Announces to all whitelisted users.")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "message", "What you'd like to say")
                .required(true),
        )
}