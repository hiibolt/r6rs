use crate::VecDeque;
use crate::Message;
use crate::Context;
use crate::unimplemented;
use crate::send_embed;

pub async fn bans( ctx: Context, msg: Message, mut args: VecDeque<String> ) {
    match args
        .pop_front()
        .unwrap_or(String::from("help"))
        .as_str()
    {
        "recent" => {
            unimplemented( ctx, msg, "recent" ).await;
        },
        "watch" => {
            unimplemented( ctx, msg, "watch" ).await;
        },
        "unwatch" => {
            unimplemented( ctx, msg, "unwatch" ).await;
        },
        "watchlist" => {
            unimplemented( ctx, msg, "unwatch" ).await;
        },
        nonexistant => {
            send_embed(
                &ctx, 
                &msg, 
                "Command does not exist", 
                &format!("The command **{nonexistant}** is not valid!\n\n**Valid commands**:\nrecent\nwatch\nunwatch\nwatchlist"), 
                "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
            ).await
                .unwrap();
        }
    }
}