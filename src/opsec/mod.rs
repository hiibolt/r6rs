use crate::VecDeque;
use crate::Message;
use crate::Context;
use crate::unimplemented;
use crate::send_embed;

async fn linked( _ctx: &Context, _msg: &Message, args: VecDeque<String> ) -> Result<String, String> {
    Ok(args.into_iter().collect::<Vec<String>>().join(" "))
}
pub async fn opsec( ctx: Context, msg: Message, mut args: VecDeque<String> ) {
    match args
        .pop_front()
        .unwrap_or(String::from("help"))
        .as_str()
    {
        "linked" => {
            unimplemented( ctx, msg, "linked" ).await;
        },
        "help" => {
            unimplemented( ctx, msg, "help" ).await;
        },
        nonexistant => {
            send_embed(
                ctx, 
                msg, 
                "Command does not exist", 
                &format!("The command **{nonexistant}** is not valid!"), 
                "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
            ).await
                .unwrap();
        }
    }
}