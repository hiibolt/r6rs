use crate::{
    helper::bot::Sendable, 
    info,
    Colorize
};

use std::{collections::HashSet, sync::Arc};

use tokio::sync::Mutex;
use tungstenite::connect;
use anyhow::{Result, Context, anyhow};


pub async fn get_and_stringify_potential_profiles( 
    usernames: &HashSet<String>, 
    sendable: Arc<Mutex<Sendable>>,
    body: &mut String,
    allow_all: bool
) -> Result<()> {
    let mut invalid_usernames = HashSet::new();
    let mut valid_usernames = HashSet::new();

    for username in usernames.iter() {
        // If the username is bad, let the user know.
        if !is_valid_sherlock_username(&username, allow_all) {
            invalid_usernames.insert(username.clone());

            continue;
        }

        valid_usernames.insert(username.clone());
    }

    // Query Sherlock
    for username in valid_usernames.iter() {
        info!("Querying Sherlock for {username}");

        *body += &format!("\n### {username}\n");

        let sherlock_ws_url = std::env::var("SHERLOCK_WS_URL")
            .expect("SHERLOCK_WS_URL not set!");
        let (
            mut socket,
            response
        ) = connect(&sherlock_ws_url)
            .context("Can't connect")?;
        let status = response.status();

        info!("Connected to Sherlock API!");
        info!("Response HTTP code: {status}");

        socket.send(tungstenite::protocol::Message::Text(format!("{username}")))
            .context("Failed to send message to Sherlock API!")?;

        // Read messages until the server closes the connection
        let mut found = false;
        loop {
            let message = socket.read().expect("Failed to read message from Sherlock API!");

            if let tungstenite::protocol::Message::Text(text) = message {
                if text.contains("http") || text.contains("https") {
                    info!("Found site for {username}: {text}");

                    found = true;

                    sendable.lock().await.add_line(
                        format!("{text}")
                    ).await
                        .map_err(|e| anyhow!("{e:?}"))?;
                }
            } else {
                break;
            }
        }

        if !found {
            sendable.lock().await.add_line(
                format!("\nNo results found for {username}")
            ).await
                .map_err(|e| anyhow!("{e:?}"))?;
        }
    }
    
    if invalid_usernames.len() > 0 {
        let mut ignored_addendum = String::new();
        ignored_addendum += "\n### Ignored Usernames\n";
        
        ignored_addendum += &invalid_usernames.into_iter()
            .map(|username| format!("- {username}"))
            .collect::<Vec<String>>()
            .join("\n");

            ignored_addendum += "\n\nThese usernames would produce poor results from Sherlock. You can always run them manually with the OSINT section :)\n`>>osint sherlock <username>`";

        sendable.lock().await.add_line(
            ignored_addendum
        ).await
            .map_err(|e| anyhow!("{e:?}"))?;
    }

    Ok(())
}
pub fn is_valid_sherlock_username ( 
    username: &str,
    allow_all: bool 
) -> bool {
    let invalid_characters: [char; 5] = [' ', '.', '-', '_', '#'];
    
    let has_no_invalid_char: bool = !invalid_characters
        .iter()
        .any(|&ch| username.contains(ch));
    let has_alpha_first: bool = username
        .chars()
        .next().unwrap_or(' ')
        .is_alphabetic();
    let within_length: bool = username.chars().count() < 20;

    // If the username is bad, let the user know.
    allow_all || ( has_no_invalid_char && has_alpha_first && within_length )
}