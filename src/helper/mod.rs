use crate::Message;
use crate::Context;
use serenity::all::{ CreateEmbed, CreateMessage };

pub async fn send_embed( ctx: Context, msg: Message, title: &str, description: &str, url: &str ) -> Result<(), String> {
    let embed = CreateEmbed::new()
    .title(title)
    .description(description)
    .thumbnail(url);
    
    let builder = CreateMessage::new().tts(true).embed(embed);

    msg.channel_id.send_message(&ctx.http, builder).await.map_err(|e| format!("{e:?}"))?;

    Ok(())
}
pub async fn unimplemented( ctx: Context, msg: Message, cmd: &str ) {
    send_embed(
        ctx, 
        msg, 
        "Not yet implemented!", 
        &format!("The command **{cmd}** exists but is not yet implemented! While I work, stay cozy :3"), 
        "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
    ).await
        .unwrap();
}
pub async fn no_access( ctx: Context, msg: Message, cmd: &str, id: u64 ) {
    send_embed(
        ctx, 
        msg, 
        "You don't have access to this command!", 
        &format!("You (**@{id}**) aren't authorized to use **{cmd}**.\n\n*Contact @hiibolt to purchase access or if this is in error.*"), 
        "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
    ).await
        .unwrap();
}