use crate::helper::get_random_anime_girl;
use crate::helper::send_embed_no_return;
use crate::helper::AsyncFnPtr;
use crate::helper::BackendHandles;
use crate::helper::R6RSCommand;
use crate::VecDeque;
use crate::Message;
use crate::helper::save;


pub async fn whitelist(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    mut args: VecDeque<String>
) -> Result<(), String> {
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
    backend_handles.state.lock().await
        .bot_data
        .get_mut("whitelisted_user_ids").ok_or(String::from("Missing whitelisted IDs JSON value!"))?
        .get_mut(section).ok_or(format!("Missing that section's JSON value!"))?
        .as_array_mut().ok_or(format!("That section isn't an array!"))?
        .push(user_id.into());
    
    // Save
    save( backend_handles.state ).await;

    send_embed_no_return(
        ctx, 
        msg, 
        "Admin - Whitelist Success", 
        &format!("Successfully added person to section!"), 
        get_random_anime_girl()
    ).await
        .map_err(|e| format!("{e:?}"))?;

    Ok(())
}
pub async fn blacklist(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    mut args: VecDeque<String>
) -> Result<(), String> {
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
    let removed_user = backend_handles.state.lock().await
        .bot_data
        .get_mut("whitelisted_user_ids").ok_or(String::from("Missing whitelisted IDs JSON value!"))?
        .get_mut(section.clone()).ok_or(format!("Missing that section's JSON value!"))?
        .as_array_mut().ok_or(format!("That section isn't an array!"))?
        .iter()
        .filter(|&val| val.as_i64().expect("Unreachable") != user_id)
        .map(|val| val.clone())
        .collect();
    (*backend_handles.state.lock().await
        .bot_data
        .get_mut("whitelisted_user_ids").ok_or(String::from("Missing whitelisted IDs JSON value!"))?
        .get_mut(section).ok_or(format!("Missing that section's JSON value!"))?
        .as_array_mut().ok_or(format!("That section isn't an array!"))?)
        = removed_user;

    // Save
    save( backend_handles.state ).await;

    println!("Admin - Blacklist Success!");
    send_embed_no_return(
        ctx, 
        msg, 
        "Success", 
        &format!("Successfully removed person from section, if they existed!"), 
        get_random_anime_girl()
    ).await.unwrap();

    Ok(())
}

pub async fn build_admin_commands() -> R6RSCommand {
    let mut admin_nest_command = R6RSCommand::new_root(
        String::from("Admin commands, generally intended only for usage by the owner."),
        String::from("Admin")
    );
    admin_nest_command.attach(
        String::from("blacklist"),
        R6RSCommand::new_leaf(
            String::from("Removes a person from the authorized user list."),
            AsyncFnPtr::new(blacklist),
            vec!(vec!(String::from("section"), String::from("user id"))),
            Some(String::from("admin"))
        )
    );
    admin_nest_command.attach(
        String::from("whitelist"),
        R6RSCommand::new_leaf(
            String::from("Adds a person to the authorized user list."),
            AsyncFnPtr::new(whitelist),
            vec!(vec!(String::from("section"), String::from("user id"))),
            Some(String::from("admin")),
        )
    );

    admin_nest_command
}