use tokio::sync::Mutex;

use crate::{
    helper::bot::{BackendHandles, Sendable},
    helper::lib::{dm_to_person, get_random_anime_girl, AsyncFnPtr, save},
    helper::command::R6RSCommand,
    info, startup, VecDeque,
    Colorize,
};

use std::{collections::HashSet, sync::Arc};

pub async fn whitelist(
    backend_handles: BackendHandles,
    sendable: Arc<Mutex<Sendable>>,
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

    tokio::spawn(async move {
        sendable.lock().await.send(
            "Admin - Whitelist Success".to_string(),
            format!("Successfully added {user_id} to section!"),
            get_random_anime_girl().to_string()
        ).await
            .expect("Failed to send message!");

        sendable.lock().await.finalize()
            .await.expect("Failed to finalize message!");
    });

    Ok(())
}
pub async fn blacklist(
    backend_handles: BackendHandles,
    sendable: Arc<Mutex<Sendable>>,
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

    tokio::spawn(async move {
        sendable.lock().await.send(
        "Admin - Blacklist Success".to_string(),
        format!("Successfully removed {user_id} from section!"),
        get_random_anime_girl().to_string()
        ).await
            .expect("Failed to send message!");

        sendable.lock().await.finalize()
            .await.expect("Failed to finalize message!");
    });

    Ok(())
}
pub async fn announce(
    backend_handles: BackendHandles,
    sendable: Arc<Mutex<Sendable>>,
    mut args: VecDeque<String>
) -> Result<(), String> {
    let mut users = HashSet::new();

    let sections_string = args.pop_front()
        .ok_or(String::from("Missing second argument `sections`!"))?;
    let sections: Vec<&str> = sections_string
        .split(",")
        .collect::<Vec<&str>>();
    let message = args
        .into_iter()
        .collect::<Vec<String>>()
        .join(" ");

    // Debug
    info!("Sending message to users in sections: {sections:?}");

    for section in sections {
        // First, get the value
        let section_value = backend_handles.state.lock().await
            .bot_data
            .get("whitelisted_user_ids").ok_or(String::from("Missing whitelisted IDs JSON value!"))?
            .get(section).ok_or(format!("Missing that section's JSON value!"))?
            .as_array().ok_or(format!("That section isn't an array!"))?
            .to_owned();

        // Next, convert the value to a list of user ids
        let section_vec = section_value
            .iter()
            .map(|val| val.as_u64().expect("Unreachable"))
            .collect::<Vec<u64>>();

        // Finally, add the user ids to the list
        users.extend(section_vec.into_iter());
    }

    // Convert the list of user ids to a list of user id objects
    let user_ids = users
        .into_iter()
        .map(|val| serenity::model::id::UserId::new(val))
        .collect::<Vec<serenity::model::id::UserId>>();

    // Finally, send the message to each user via DM
    for user_id in user_ids {
        info!("Sending message to user: {user_id:?}");

        let guard = sendable.lock().await;
        match *guard {
            Sendable::DiscordResponseSender(ref inner) => {
                // DM the user
                let message = message.clone();
                let ctx = inner.ctx.clone();
                tokio::spawn(async move {
                    dm_to_person(
                        ctx,
                        user_id,
                        message
                    )
                        .await.map_err(|e| format!("{e:?}"))
                        .expect("Failed to send message!");
                });
            },
            _ => {
                return Err(String::from("This command can only be used in Discord!"));
            }
        }
    }

    tokio::spawn(async move {
        sendable.lock().await
            .finalize()
            .await.expect("Failed to finalize message!");
    });
    
    Ok(())
}
pub async fn dm_person(
    _backend_handles: BackendHandles,
    sendable: Arc<Mutex<Sendable>>,
    mut args: VecDeque<String>
) -> Result<(), String> {
    // Extract both the message and the user id
    let user_id = args.pop_front()
        .ok_or(String::from("Missing first argument `user id`!"))?
        .parse::<u64>()
        .map_err(|_| String::from("Suppiled Discord User ID ust be an integer!"))?;
    let message = args
        .into_iter()
        .collect::<Vec<String>>()
        .join(" ");

    // Debug
    info!("Sending message to user: {user_id:?}");

    // Send the message
    let guard = sendable.lock().await;
    match *guard {
        Sendable::DiscordResponseSender(ref inner) => {
            // DM the user
            let ctx = inner.ctx.clone();

            tokio::spawn(async move {
                dm_to_person(
                    ctx,
                    serenity::model::id::UserId::new(user_id),
                    message
                )
                    .await.map_err(|e| format!("{e:?}"))
                    .expect("Failed to send message!");
            });
        }
        _ => {
            return Err(String::from("This command can only be used in Discord!"));
        }
    }

    // Send a confirmation message
    let copied_sendable = sendable.clone();
    tokio::spawn(async move { 
        copied_sendable.lock().await.send(
            "Admin - DM Success".to_string(),
            format!("Successfully sent message to user!"),
            get_random_anime_girl().to_string()
        ).await
            .map_err(|e| format!("{e:?}"))
            .expect("Failed to send message!");

        copied_sendable.lock().await.finalize()
            .await.expect("Failed to finalize message!");
    });
    
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
    admin_nest_command.attach(
        String::from("announce"),
        R6RSCommand::new_leaf(
            String::from("Announces a message to all whitelisted users."),
            AsyncFnPtr::new(announce),
            vec!(vec!(String::from("sections"), String::from("message"))),
            Some(String::from("admin")),
        )
    );
    admin_nest_command.attach(
        String::from("dm"),
        R6RSCommand::new_leaf(
            String::from("DMs a message to a specific user."),
            AsyncFnPtr::new(dm_person),
            vec!(vec!(String::from("user id"), String::from("message"))),
            Some(String::from("admin")),
        )
    );

    startup!("Admin commands have been built.");

    admin_nest_command
}