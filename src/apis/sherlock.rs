use std::collections::HashSet;
use serenity::all::Message;
use tungstenite::connect;
use crate::helper::edit_embed;


pub async fn get_and_stringify_potential_profiles( 
    usernames: &HashSet<String>, 
    ctx: &serenity::client::Context, 
    msg: &mut Message, 
    title: &str, 
    body: &mut String, 
    url: &str,
    no_special_characters: bool
) {
    let mut invalid_usernames = HashSet::new();
    let mut valid_usernames = HashSet::new();

    for username in usernames.iter() {
        // If the username is bad, let the user know.
        if !is_valid_sherlock_username(&username, no_special_characters) {
            invalid_usernames.insert(username.clone());

            continue;
        }

        valid_usernames.insert(username.clone());
    }

    // Query Sherlock
    for username in valid_usernames.iter() {
        println!("Querying Sherlock for {username}");

        *body += &format!("\n### {username}\n");

        let sherlock_ws_url = std::env::var("SHERLOCK_WS_URL")
            .expect("SHERLOCK_WS_URL not set!");
        let (mut socket, response) = connect(&sherlock_ws_url)
            .expect("Can't connect");

        println!("Connected to Sherlock API!");
        println!("Response HTTP code: {}", response.status());

        socket.send(tungstenite::protocol::Message::Text(format!("{username}")))
            .expect("Failed to send message to Sherlock API!");

        // Read messages until the server closes the connection
        let mut found = false;
        loop {
            let message = socket.read().expect("Failed to read message from Sherlock API!");

            if let tungstenite::protocol::Message::Text(text) = message {
                if text.contains("http") || text.contains("https") {
                    println!("Found site for {username}: {text}");

                    found = true;

                    *body += &format!("{text}");            
                    edit_embed(
                        &ctx,
                        msg,
                        title,
                        &body,
                        url
                    ).await;
                }
            } else {
                break;
            }
        }

        if !found {
            *body += &format!("\nNo results found for {username}");
            edit_embed(
                &ctx,
                msg,
                title,
                &body,
                url
            ).await;
        }
    }
    
    if invalid_usernames.len() > 0 {
        *body += "\n### Ignored Usernames\n";
        
        *body += &invalid_usernames.into_iter()
            .map(|username| format!("- {username}"))
            .collect::<Vec<String>>()
            .join("\n");

        *body += "\n\nThese usernames would produce poor results from Sherlock. You can always run them manually with the OSINT section :)\n`>>osint sherlock <username>`";

        edit_embed(
            &ctx,
            msg,
            title,
            &body,
            url
        ).await;
    }
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