use crate::VecDeque;
use crate::Message;
use crate::Context;
use crate::send_embed;
use crate::State;
use crate::Value;
use crate::{ Arc, Mutex };
use crate::helper::save;

pub async fn whitelist( state: Arc<Mutex<State>>, mut args: VecDeque<String> ) -> Result<(), String> {
    // Get the input
    let section = args
        .pop_front()
        .ok_or(String::from("Missing `section`!"))?;
    let user_id = args
        .pop_front()
        .ok_or(String::from("Missing `user id`!"))?
        .parse::<u64>()
        .map_err(|_| String::from("Suppiled Discord User ID ust be an integer!"))?;

    // Update the entry
    state.lock().await
        .bot_data
        .get_mut("whitelisted_user_ids").ok_or(String::from("Missing whitelisted IDs JSON value!"))?
        .get_mut(section).ok_or(format!("Missing that section's JSON value!"))?
        .as_array_mut().ok_or(format!("That section isn't an array!"))?
        .push(user_id.into());
    
    // Save
    save( state ).await;

    Ok(())
}
pub async fn blacklist( state: Arc<Mutex<State>>, mut args: VecDeque<String> ) -> Result<(), String> {
    // Get the input
    let section = args
        .pop_front()
        .ok_or(String::from("Missing `section`!"))?;
    let user_id = args
        .pop_front()
        .ok_or(String::from("Missing `user id`!"))?
        .parse::<i64>()
        .map_err(|_| String::from("Suppiled Discord User ID ust be an integer!"))?;

    // Update the entry
    let removed_user = state.lock().await
        .bot_data
        .get_mut("whitelisted_user_ids").ok_or(String::from("Missing whitelisted IDs JSON value!"))?
        .get_mut(section.clone()).ok_or(format!("Missing that section's JSON value!"))?
        .as_array_mut().ok_or(format!("That section isn't an array!"))?
        .iter()
        .filter(|&val| val.as_i64().expect("Unreachable") != user_id)
        .map(|val| val.clone())
        .collect();
    (*state.lock().await
        .bot_data
        .get_mut("whitelisted_user_ids").ok_or(String::from("Missing whitelisted IDs JSON value!"))?
        .get_mut(section).ok_or(format!("Missing that section's JSON value!"))?
        .as_array_mut().ok_or(format!("That section isn't an array!"))?)
        = removed_user;

    // Save
    save( state ).await;

    Ok(())
}
pub async fn help( ctx: Context, msg: Message ) {
    send_embed(
        &ctx, 
        &msg, 
        "ADMIN - Help", 
        &format!("**Command List**:\n- `r6 admin whitelist <user id>`\n- `r6 admin blacklist <user id>`\n- `r6 admin help`"), 
        "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
    ).await.unwrap();
}
pub async fn admin( state: Arc<Mutex<State>>, ctx: Context, msg: Message, mut args: VecDeque<String> ) {
    match args
        .pop_front()
        .unwrap_or(String::from("help"))
        .as_str()
    {
        "whitelist" => {
            tokio::spawn(async move {
                match whitelist( state, args ).await {
                    Ok(_) => {
                        println!("Success!");
                        send_embed(
                            &ctx, 
                            &msg, 
                            "ADMIN - Whitelist Success", 
                            &format!("Successfully added person to section!"), 
                            "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
                        ).await.unwrap();
                    },
                    Err(err) => {
                        println!("Failed! [{err}]");
                        send_embed(
                            &ctx, 
                            &msg, 
                            "ADMIN - Whitelist Error", 
                            &format!("Failed for reason `{err}`"), 
                            "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
                        ).await.unwrap();
                    }
                }
            });
        },
        "blacklist" => {
            tokio::spawn(async move {
                match blacklist( state, args ).await {
                    Ok(_) => {
                        println!("ADMIN - Blacklist Success!");
                        send_embed(
                            &ctx, 
                            &msg, 
                            "Success", 
                            &format!("Successfully removed person from section, if they existed!"), 
                            "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
                        ).await.unwrap();
                    },
                    Err(err) => {
                        println!("Failed! [{err}]");
                        send_embed(
                            &ctx, 
                            &msg, 
                            "ADMIN - Blacklist Error", 
                            &format!("Failed for reason `{err}`"), 
                            "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
                        ).await.unwrap();
                    }
                }
            });
        },
        "help" => {
            tokio::spawn(help( ctx, msg ));
        }
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