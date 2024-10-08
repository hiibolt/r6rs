use crate::{
    apis::{is_valid_sherlock_username, Snusbase},
    helper::{
        lib::{get_random_anime_girl, AsyncFnPtr},
        bot::{BackendHandles, Sendable}, 
        command::R6RSCommand
    },
    info, startup,
    Colorize
};

use std::{collections::VecDeque, sync::Arc};

use serenity::all::CreateMessage;
use tokio::sync::Mutex;
use tungstenite::connect;

pub async fn lookup( 
    snusbase: Arc<Mutex<Snusbase>>,
    sendable: Arc<Mutex<Sendable>>,
    mut args: VecDeque<String>,
    lookup_type: &str
) -> Result<(), String> {
    // First, load the blacklisted strings from ./assets/blacklist.txt
    let blacklisted_strings = std::fs::read_to_string("./assets/blacklist.txt")
        .expect("Failed to read blacklist.txt!")
        .split("\n")
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let mut cloned_args = args.clone();
    let first_arg = cloned_args.pop_front();

    // Check if the query contains any blacklisted strings
    for blacklisted_string in &blacklisted_strings {
        if let Some(ref ar) = first_arg {
            if *ar == *blacklisted_string {
                return Err(format!("The query contains a blacklisted string: '{}'. If this is in error, please contact @hiibolt!", blacklisted_string));
            }
        }
    }

    let snusbase_response_result = match lookup_type {
        "email" => {
            let mut ret = Err(anyhow::anyhow!("No email provided!"));

            if let Some(email) = args.pop_front() {
                ret = snusbase.lock()
                    .await
                    .get_by_email(email)
                    .await;
            }
            
            ret
        },
        "username" => {
            let mut ret = Err(anyhow::anyhow!("No username provided!"));

            if let Some(username) = args.pop_front() {
                ret = snusbase.lock()
                    .await
                    .get_by_username(username)
                    .await;
            }
            
            ret
        },
        "password" => {
            let mut ret = Err(anyhow::anyhow!("No password provided!"));

            if let Some(password) = args.pop_front() {
                ret = snusbase.lock()
                    .await
                    .get_by_password(password)
                    .await;
            }
            
            ret
        },
        "last_ip" => {
            let mut ret = Err(anyhow::anyhow!("No last IP provided!"));

            if let Some(last_ip) = args.pop_front() {
                ret = snusbase.lock()
                    .await
                    .get_by_last_ip(last_ip)
                    .await;
            }

            ret
        }
        "name" => {
            let mut ret = Err(anyhow::anyhow!("No name provided!"));

            if args.len() > 0 {
                ret = snusbase.lock()
                    .await
                    .get_by_name(args.into_iter().collect::<Vec<String>>().join(" "))
                    .await;
            }
            
            ret
        },
        "hash" => {
            let mut ret = Err(anyhow::anyhow!("No hash provided!"));

            if args.len() > 0 {
                ret = snusbase.lock()
                    .await
                    .get_by_hash(args.into_iter().collect::<Vec<String>>().join(" "))
                    .await;
            }

            ret
        },
        _ => { panic!("Invalid lookup type!"); }
    };

    if snusbase_response_result.is_err() {
        return Err(format!("{}", snusbase_response_result.unwrap_err()));
    }

    let snusbase_response = snusbase_response_result.expect("unreachable");
    
    let mut number_of_entries: i32 = 0;
    for map in snusbase_response.results.values() {
        for _ in map.iter() {
            number_of_entries += 1;
        }
    }

    if number_of_entries > 10 {
        let full_dump = format!("{}", snusbase_response);

        let copied_sendable = sendable.clone();
        tokio::spawn(async move {
            copied_sendable.lock().await.send(
                "OSINT DUMP".to_string(),
                "There were more than 10 results, which in total contains more data than Discord can display.\n\nA full dump will be attached below shortly!".to_string(),
                get_random_anime_girl().to_string()
            ).await.expect("Failed to send to sendable!");

            copied_sendable.lock().await
                .finalize()
                .await.expect("Failed to finalize message!");
        });

        let builder = CreateMessage::new();

        // This command only works on Discord, for now.
        tokio::spawn(async move {
            sendable.lock().await
                .send_text_file(
                    full_dump,
                    builder
                ).await
                    .expect("Failed to upload file!");
        });

        return Ok(());
    }

    if snusbase_response.results.len() == 0 {
        tokio::spawn(async move {
            sendable.lock().await.send(
                "No results".to_string(),
                "Nothing was found for the given query!\n\n*There were no errors, but there weren't any results either.*".to_string(),
                get_random_anime_girl().to_string()
            ).await.expect("Failed to send to sendable!");

            sendable.lock().await
                .finalize()
                .await.expect("Failed to finalize message!");
        });
        
        return Ok(());
    }

    let number_of_sources = snusbase_response.results.len();
    for (ind, (dump, content)) in snusbase_response.results.iter().take(10).enumerate() {
        let number_of_dumps = content.len();
        
        let mut dump_ind = 0;
        for entry in content {
            let mut message = String::new();
            dump_ind += 1;
            
            message += &format!("## Source {}/{number_of_sources} - Dump {dump_ind}/{number_of_dumps}\n", ind + 1);

            for (key, value) in entry {
                message += &format!("- **{}**: {}\n", key, value);
            }
            
            message += &format!("\n(From `{}`):\n", dump);

            
            let copied_sendable = sendable.clone();
            tokio::spawn(async move {
                copied_sendable.lock().await.send(
                    "OSINT DUMP".to_string(),
                    message,
                    get_random_anime_girl().to_string()
                ).await.expect("Failed to send to sendable!");

                copied_sendable.lock().await
                    .finalize()
                    .await.expect("Failed to finalize message!");
            });
        }
    }

    Ok(())
}

pub async fn query_email(
    backend_handles: BackendHandles,
    sendable: Arc<Mutex<Sendable>>,
    args: VecDeque<String>
) -> Result<(), String> {
    lookup(backend_handles.snusbase, sendable, args, "email").await
}
pub async fn query_username(
    backend_handles: BackendHandles,
    sendable: Arc<Mutex<Sendable>>,
    args: VecDeque<String>
) -> Result<(), String> {
    lookup(backend_handles.snusbase, sendable, args, "username").await
}
pub async fn query_last_ip(
    backend_handles: BackendHandles,
    sendable: Arc<Mutex<Sendable>>,
    args: VecDeque<String>
) -> Result<(), String> {
    lookup(backend_handles.snusbase, sendable, args, "last_ip").await
}
pub async fn query_hash(
    backend_handles: BackendHandles,
    sendable: Arc<Mutex<Sendable>>,
    args: VecDeque<String>
) -> Result<(), String> {
    lookup(backend_handles.snusbase, sendable, args, "hash").await
}
pub async fn query_password(
    backend_handles: BackendHandles,
    sendable: Arc<Mutex<Sendable>>,
    args: VecDeque<String>
) -> Result<(), String> {
    lookup(backend_handles.snusbase, sendable, args, "password").await
}
pub async fn query_name(
    backend_handles: BackendHandles,
    sendable: Arc<Mutex<Sendable>>,
    args: VecDeque<String>
) -> Result<(), String> {
    lookup(backend_handles.snusbase, sendable, args, "name").await
}
pub async fn cnam_lookup(
    backend_handles: BackendHandles,
    sendable: Arc<Mutex<Sendable>>,
    mut args: VecDeque<String>
) -> Result<(), String> {
    // First, load the blacklisted strings from ./assets/blacklist.txt
    let blacklisted_strings = std::fs::read_to_string("./assets/blacklist.txt")
        .expect("Failed to read blacklist.txt!")
        .split("\n")
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let mut cloned_args = args.clone();
    let first_arg = cloned_args.pop_front();

    // Check if the query contains any blacklisted strings
    for blacklisted_string in &blacklisted_strings {
        if let Some(ref ar) = first_arg {
            if *ar == *blacklisted_string {
                return Err(format!("The query contains a blacklisted string: '{}'. If this is in error, please contact @hiibolt!", blacklisted_string));
            }
        }
    }
    let phone_number = args.pop_front()
        .ok_or(String::from("Missing phone number!"))?;

    let response = backend_handles.bulkvs.lock()
        .await
        .query_phone_number(&phone_number
            .replace("-", "")
            .replace("(", "")
            .replace(")", ""))
        .map_err(|e| format!("{e:#?}"))?;

    let mut message = String::new();
    if let Some(name) = response.name {
        message += &format!("\n- **Name**: {name}");
    }
    if let Some(number) = response.number {
        message += &format!("\n- **Number**: {number}");
    }
    if let Some(time) = response.time {
        message += &format!("\n- **Time**: {time}");
    }

    tokio::spawn(async move {
        sendable.lock().await.send(
            "CNAM Lookup".to_string(),
            message,
            get_random_anime_girl().to_string()
        ).await.expect("Failed to send to sendable!");

        sendable.lock().await
            .finalize()
            .await.expect("Failed to finalize message!");
    });

    Ok(())
}
async fn geolocate(
    backend_handles: BackendHandles,
    sendable: Arc<Mutex<Sendable>>,
    args: VecDeque<String>
) -> Result<(), String> {
    let response = backend_handles.snusbase.lock()
        .await
        .whois_ip_query(args.into_iter().collect())
        .await
        .map_err(|e| format!("{e:#?}"))?;


    let mut message = String::new();
    for (ip, content) in &response.results {
        message += &format!("## IP (*{}*):\n", ip);

        for (key, value) in content {
            if value.is_string() {
                message += &format!("\n- **{}**: {:?}", key, value.as_str().unwrap());
            } else if value.is_number() {
                message += &format!("\n- **{}**: {:?}", key, value.as_number().unwrap());
            } else {
                message += &format!("\n- **{}**: {:?}", key, value);
            }
        }
    }

    tokio::spawn(async move {
        sendable.lock().await.send(
            "IP Lookup".to_string(),
            message,
            get_random_anime_girl().to_string()
        ).await.expect("Failed to send to sendable!");

        sendable.lock().await
            .finalize()
            .await.expect("Failed to finalize message!");
    });

    Ok(())
}
pub async fn dehash(
    backend_handles: BackendHandles,
    sendable: Arc<Mutex<Sendable>>,
    args: VecDeque<String>
) -> Result<(), String> {
    let response = backend_handles.snusbase.lock()
        .await
        .dehash(args.into_iter().collect())
        .await
        .map_err(|e| format!("{e:#?}"))?;

    let mut body = String::new();
    let number_of_dumps = response.results.len();
    let mut total_results = 0;

    for (dump_ind, ( dump_name, content)) in response.results.iter().enumerate() {
        body += &format!("## Dump {}/{}:\n*(From `{}`)*\n", dump_ind + 1, number_of_dumps, dump_name);

        let number_of_results_in_dump = content.len();
        for (result_ind, value) in content.iter().enumerate() {
            total_results += 1;

            for (key, value) in value.as_object().expect("Didn't get an object back from backend?") {
                body += &format!("\nResult {}/{}:\n- **{}**: {}\n", result_ind + 1, number_of_results_in_dump, key, value);
            }
        }
    }

    if total_results > 20 {
        let copied_sendable = sendable.clone();
        tokio::spawn(async move {
            copied_sendable.lock().await.send(
                "OSINT DUMP - `dehash`".to_string(),
                "There were more than 20 results, which in total contains more data than Discord can display.\n\nA full dump will be attached below shortly!".to_string(),
                get_random_anime_girl().to_string()
            ).await.expect("Failed to send to sendable!");

            copied_sendable.lock().await
                .finalize()
                .await.expect("Failed to finalize message!");
        });

        let builder = CreateMessage::new();

        // This command only works on Discord, for now.
        tokio::spawn(async move {
            sendable.lock().await
                .send_text_file(
                    body,
                    builder
                ).await
                    .expect("Failed to upload file!");
        });

        return Ok(());
    } else if total_results == 0 {
        tokio::spawn(async move {
            sendable.lock().await.send(
                "No results".to_string(),
                "There were no errors, but there were also no results!".to_string(),
                get_random_anime_girl().to_string()
            ).await.expect("Failed to send to sendable!");

            sendable.lock().await
                .finalize()
                .await.expect("Failed to finalize message!");
        });

        return Ok(());
    }

    tokio::spawn(async move {
        sendable.lock().await.send(
            "Dehash Results".to_string(),
            body,
            get_random_anime_girl().to_string()
        ).await.expect("Failed to send to sendable!");

        sendable.lock().await
            .finalize()
            .await.expect("Failed to finalize message!");
    });

    Ok(())
}
pub async fn rehash(
    backend_handles: BackendHandles,
    sendable: Arc<Mutex<Sendable>>,
    args: VecDeque<String>
) -> Result<(), String> {
    let response = backend_handles.snusbase.lock()
        .await
        .rehash(args.into_iter().collect())
        .await
        .map_err(|e| format!("{e:#?}"));

    let mut body = String::new();
    let response = response.unwrap();
    let number_of_dumps = response.results.len();
    let mut total_results = 0;

    for (dump_ind, ( dump_name, content)) in response.results.iter().enumerate() {
        body += &format!("## Dump {}/{}:\n*(From `{}`)*\n", dump_ind + 1, number_of_dumps, dump_name);

        let number_of_results_in_dump = content.len();
        for (result_ind, value) in content.iter().enumerate() {
            total_results += 1;

            for (key, value) in value.as_object().expect("Didn't get an object back from backend?") {
                body += &format!("\nResult {}/{}:\n- **{}**: {}\n", result_ind + 1, number_of_results_in_dump, key, value);
            }
        }
    }

    if total_results > 20 {
        let copied_sendable = sendable.clone();
        tokio::spawn(async move {
            copied_sendable.lock().await.send(
                "OSINT DUMP - `rehash`".to_string(),
                "There were more than 20 results, which in total contains more data than Discord can display.\n\nA full dump will be attached below shortly!".to_string(),
                get_random_anime_girl().to_string()
            ).await.expect("Failed to send to sendable!");

            copied_sendable.lock().await
                .finalize()
                .await.expect("Failed to finalize message!");
        });

        let builder = CreateMessage::new();

        // This command only works on Discord, for now.
        tokio::spawn(async move {
            sendable.lock().await
                .send_text_file(
                    body,
                    builder
                ).await
                    .expect("Failed to upload file!");
        });

        return Ok(());
    } else if total_results == 0 {
        tokio::spawn(async move {
            sendable.lock().await.send(
                "No results".to_string(),
                "There were no errors, but there were also no results!".to_string(),
                get_random_anime_girl().to_string()
            ).await.expect("Failed to send to sendable!");

            sendable.lock().await
                .finalize()
                .await.expect("Failed to finalize message!");
        });

        return Ok(());
    }

    tokio::spawn(async move {
        sendable.lock().await.send(
            "Rehash Results".to_string(),
            body,
            get_random_anime_girl().to_string()
        ).await.expect("Failed to send to sendable!");

        sendable.lock().await
            .finalize()
            .await.expect("Failed to finalize message!");
    });

    Ok(())
}
pub async fn sherlock_helper(
    username: String,
    
    sendable: Arc<Mutex<Sendable>>
) {
    let mut body = String::new();
    // Warn the user if the username is poor quality
    if !is_valid_sherlock_username(&username, false) {              
        body += &format!("### Warning\nThe username `{username}` has special characters and may not return quality results!");
    }

    let title = format!("OSINT - Sherlock - {username}");
    let url = get_random_anime_girl();
    
    sendable.lock().await.send(
        title,
        "## Results\n".to_string(),
        url.to_string()
    ).await.expect("Failed to send to sendable!");
    
    // Query Sherlock
    info!("Querying Sherlock for {username}");

    body += &format!("\n### {username}\n");

    let sherlock_ws_url = std::env::var("SHERLOCK_WS_URL")
        .expect("SHERLOCK_WS_URL not set!");
    let (mut socket, response) = connect(&sherlock_ws_url)
        .expect("Can't connect");
    let response_code = &response.status();

    info!("Connected to Sherlock API!");
    info!("Response HTTP code: `{response_code}`");

    socket.send(tungstenite::protocol::Message::Text(format!("{username}")))
        .expect("Failed to send message to Sherlock API!");

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
                ).await.expect("Failed to send to sendable!");
            }
        } else {
            break;
        }
    }

    if !found {
        sendable.lock().await.add_line(
            format!("\nNo results found for {username}")
        ).await.expect("Failed to send to sendable!");
    }

    sendable.lock().await
        .finalize()
        .await.expect("Failed to finalize message!");
}
pub async fn sherlock(
    _backend_handles: BackendHandles,
    sendable: Arc<Mutex<Sendable>>,
    mut args: VecDeque<String>
) -> Result<(), String> {
    let username = args.pop_front()
        .ok_or(String::from("Please provide a username!"))?;

    tokio::spawn(sherlock_helper(username, sendable));

    Ok(())
}

pub async fn build_osint_commands() -> R6RSCommand {

    let mut osint_nest_command = R6RSCommand::new_root(
        String::from("Commands for gather Open Source Intelligence (OSINT). Please see the GitHub for Terms of Usage."),
        String::from("OSINT")
    );
    // Create a nest for query-based commands
    let mut query_nest_command = R6RSCommand::new_root(
        String::from("Query-based commands for OSINT."),
        String::from("Queries")
    );
    
    // Attach the query-based commands to the query nest
    query_nest_command.attach(
        String::from("email"),
        R6RSCommand::new_leaf(
            String::from("Queries for leaks based on an email."),
            AsyncFnPtr::new(query_email),
            vec!(vec!(String::from("email"))),
            Some(String::from("osint"))
        )
    );
    query_nest_command.attach(
        String::from("username"),
        R6RSCommand::new_leaf(
            String::from("Queries for leaks based on a username."),
            AsyncFnPtr::new(query_username),
            vec!(vec!(String::from("username"))),
            Some(String::from("osint"))
        )
    );
    query_nest_command.attach(
        String::from("ip"),
        R6RSCommand::new_leaf(
            String::from("Queries for leaks based on a last IP."),
            AsyncFnPtr::new(query_last_ip),
            vec!(vec!(String::from("ip"))),
            Some(String::from("osint"))
        )
    );
    query_nest_command.attach(
        String::from("hash"),
        R6RSCommand::new_leaf(
            String::from("Queries for leaks based on a hash."),
            AsyncFnPtr::new(query_hash),
            vec!(vec!(String::from("hash"))),
            Some(String::from("osint"))
        )
    );
    query_nest_command.attach(
        String::from("password"),
        R6RSCommand::new_leaf(
            String::from("Queries for leaks based on a password."),
            AsyncFnPtr::new(query_password),
            vec!(vec!(String::from("password"))),
            Some(String::from("osint"))
        )
    );
    query_nest_command.attach(
        String::from("name"),
        R6RSCommand::new_leaf(
            String::from("Queries for leaks based on a name."),
            AsyncFnPtr::new(query_name),
            vec!(vec!(String::from("name"))),
            Some(String::from("osint"))
        )
    );

    // Finally, attach the query nest to the main nest
    osint_nest_command.attach(
        String::from("query"),
        query_nest_command
    );

    // Create the nest for hash-based commands
    let mut hash_nest_command = R6RSCommand::new_root(
        String::from("Hash-based commands for OSINT."),
        String::from("Hashing")
    );

    // Attach the hash-based commands to the hash nest
    hash_nest_command.attach(
        String::from("dehash"),
        R6RSCommand::new_leaf(
            String::from("Dehashes a hash into pre-cracked passwords."),
            AsyncFnPtr::new(dehash),
            vec!(vec!(String::from("hash"))),
            Some(String::from("osint"))
        )
    );
    hash_nest_command.attach(
        String::from("rehash"),
        R6RSCommand::new_leaf(
            String::from("Rehashes a password into pre-hashed hashes."),
            AsyncFnPtr::new(rehash),
            vec!(vec!(String::from("password"))),
            Some(String::from("osint"))
        )
    );

    // Finally, attach the hash nest to the main nest
    osint_nest_command.attach(
        String::from("hash"),
        hash_nest_command
    );

    // Other commands
    osint_nest_command.attach(
        String::from("phone"),
        R6RSCommand::new_leaf(
            String::from("Perform a Caller ID lookup on a phone number."),
            AsyncFnPtr::new(cnam_lookup),
            vec!(vec!(String::from("phone number"))),
            Some(String::from("osint"))
        )
    );
    osint_nest_command.attach(
        String::from("geolocate"),
        R6RSCommand::new_leaf(
            String::from("Geolocates an IP."),
            AsyncFnPtr::new(geolocate),
            vec!(vec!(String::from("ip"))),
            Some(String::from("osint"))
        )
    );
    osint_nest_command.attach(
        String::from("sherlock"),
        R6RSCommand::new_leaf(
            String::from("Cross-references sites with a given username."),
            AsyncFnPtr::new(sherlock),
            vec!(vec!(String::from("username"))),
            Some(String::from("osint"))
        )
    );

    startup!("OSINT commands have been built.");

    osint_nest_command
}