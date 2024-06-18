use std::sync::Arc;
use serenity::all::{CommandOptionType, Context, CreateCommandOption, ResolvedValue};
use tokio::sync::Mutex;

use serenity::builder::CreateCommand;
use serenity::model::application::ResolvedOption;

use crate::apis::Ubisoft;

pub async fn run<'a>(
    options: Vec<ResolvedOption<'a>>,
    _ctx:      &Context,
    state:    Arc<Mutex<Ubisoft>>
) -> Result<String, serenity::Error>  {
    let username = match options.get(0).expect("Missing username!").value {
        ResolvedValue::String(ref username) => username,
        _ => panic!("Username must be a string!")
    };

    let my_profile_id = state.lock().await
        .get_account_id(username.to_string(), String::from("uplay")).await
        .expect("Failed to get profile id!");

    println!("My profile id: {my_profile_id}");

    let res = state.lock().await.get_applications(my_profile_id).await
        .expect("Failed to get applications!");

    println!("Result: {res}");

    Ok(format!("{:?}",res))
}
pub fn register() -> CreateCommand {
    CreateCommand::new("development")
        .description("Can do a lot of things. Only for dev purposes.")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "message", "What you'd like to say")
                .required(true),
        )
}