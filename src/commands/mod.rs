use serenity::all::{ Context, CreateMessage };
use colored::Colorize;

use crate::{error, info};

pub mod announce_all;
pub mod announce_econ;
pub mod announce_opsec;
pub mod announce_osint;
pub mod development;

async fn dm_to_person (
    ctx: Context,
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