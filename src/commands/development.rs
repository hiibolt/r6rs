use std::sync::Arc;
use serenity::all::{ CommandOptionType, Context, CreateCommandOption };
use tokio::sync::Mutex;

use serenity::builder::CreateCommand;
use serenity::model::application::ResolvedOption;

use crate::apis::Ubisoft;


pub async fn run<'a>(
    _options: Vec<ResolvedOption<'a>>,
    _ctx:      &Context,
    _state:    Arc<Mutex<Ubisoft>>
) -> Result<String, serenity::Error>  {
    Ok(String::from("This command is not yet implemented."))
}
pub fn register() -> CreateCommand {
    CreateCommand::new("development")
        .description("Can do a lot of things. Only for dev purposes.")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "message", "What you'd like to say")
                .required(true),
        )
}