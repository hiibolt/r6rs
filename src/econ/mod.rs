use crate::VecDeque;
use crate::Message;
use crate::Context;
use crate::State;
use crate::Mutex;
use crate::Arc;
use crate::Value;
use crate::unimplemented;
use crate::send_embed;
use std::collections::HashMap;

async fn list( state: Arc<Mutex<State>> ) -> String {
    let mut msg: String = String::from(
        "# Ask Bolt for new items.\n\n## Skins:"
    );
    
    let mut count: u8 = 0;
    for (key, value) in  state.lock().await.id_list.iter() {
        // Break if we're potentially reaching Discord Embed's max length
        if ( count > 99 ){
            msg += "...plus many others!";
            break;
        }

        msg += &format!("{key}\n");

        count += 1;
    }

    msg
}
pub async fn econ( state: Arc<Mutex<State>>, ctx: Context, msg: Message, mut args: VecDeque<String> ) {
    match args
        .pop_front()
        .unwrap_or(String::from("help"))
        .as_str()
    {
        "list" => {
            let result: String = list( state ).await;

            send_embed(
                ctx, 
                msg, 
                "Tracked Skins", 
                &result, 
                "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
            ).await
                .unwrap();
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