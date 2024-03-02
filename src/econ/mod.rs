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

async fn list( state: Arc<Mutex<State>> ) -> Result<String, String> {
    let msg: String = String::new();
    let id_array = state
        .lock().await
        .id_list
        .as_array().expect("Couldn't convert to array!");
    
    let count: u8 = 0;
    for (key, value) in id_array.iter() {
        if ( count > 99 ){
            msg += "\nPlus some others";
            break;
        }
        println!("{key:?} {value:?}");
        msg += &format!("{key}\n");
        count += 1;
    }

    Ok(msg)
    //embed=discord.Embed(title=f'Tracked Skins', description=f'# Ask Bolt for new Items.\n\n# Skins:\n{msg}', color=0xFF5733)
    //embed.set_thumbnail(url="https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4")
    //await message.channel.send(embed=embed)
}
pub async fn econ( state: Arc<Mutex<State>>, ctx: Context, msg: Message, mut args: VecDeque<String> ) {
    match args
        .pop_front()
        .unwrap_or(String::from("help"))
        .as_str()
    {
        "list" => {
            let result: Result<String, String> = list( state ).await;

            match result {
                Ok(res) => {
                    send_embed(
                        ctx, 
                        msg, 
                        "Tracked Skins", 
                        &res, 
                        "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
                    ).await
                        .unwrap();
                },
                Err(err) => {
                    send_embed(
                        ctx, 
                        msg, 
                        "Error", 
                        &format!("Encountered an error while running **list**:\n{err}"), 
                        "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
                    ).await
                        .unwrap();
                }
            }
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