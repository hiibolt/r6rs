use crate::apis::{is_valid_sherlock_username, Snusbase};
use crate::helper::{ edit_embed, get_random_anime_girl, send_embed, send_embed_no_return, AsyncFnPtr, BackendHandles, R6RSCommand };

use serenity::all::{CreateAttachment, CreateMessage, Message};
use tokio::sync::Mutex;
use tungstenite::connect;
use std::{collections::VecDeque, sync::Arc};



pub async fn lookup( 
    snusbase: Arc<Mutex<Snusbase>>,
    ctx: serenity::client::Context,
    msg: Message,
    mut args: VecDeque<String>,
    lookup_type: &str
) -> Result<(), String> {
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

        send_embed_no_return(
            ctx.clone(), 
            msg.clone(), 
            "OSINT DUMP", 
            "There were more than 10 results, which in total contains more data than Discord can display.\n\nA full dump will be attached below shortly!", 
            get_random_anime_girl()
        ).await
            .unwrap();

        let builder = CreateMessage::new();

        tokio::spawn(msg.channel_id.send_files(
            ctx.http,
            std::iter::once(CreateAttachment::bytes(
                full_dump.as_bytes(),
                "full_dump.txt"
            )),
            builder
        ));

        return Ok(());
    }

    if snusbase_response.results.len() == 0 {
        send_embed_no_return(
            ctx.clone(), 
            msg.clone(), 
            "No results", 
            "Nothing was found for the given query!\n\n*There were no errors, but there weren't any results either.*", 
            get_random_anime_girl()
            ).await
                .unwrap();
        
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

            send_embed_no_return(
                ctx.clone(), 
                msg.clone(), 
                "OSINT DUMP - Via Email", 
                &message, 
                get_random_anime_girl()
            ).await
                .unwrap();
        }
    }

    Ok(())
}

pub async fn query_email(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String>
) -> Result<(), String> {
    lookup(backend_handles.snusbase, ctx, msg, args, "email").await
}
pub async fn query_username(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String>
) -> Result<(), String> {
    lookup(backend_handles.snusbase, ctx, msg, args, "username").await
}
pub async fn query_last_ip(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String>
) -> Result<(), String> {
    lookup(backend_handles.snusbase, ctx, msg, args, "last_ip").await
}
pub async fn query_hash(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String>
) -> Result<(), String> {
    lookup(backend_handles.snusbase, ctx, msg, args, "hash").await
}
pub async fn query_password(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String>
) -> Result<(), String> {
    lookup(backend_handles.snusbase, ctx, msg, args, "password").await
}
pub async fn query_name(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String>
) -> Result<(), String> {
    lookup(backend_handles.snusbase, ctx, msg, args, "name").await
}
pub async fn cnam_lookup(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    mut args: VecDeque<String>
) -> Result<(), String> {
    let phone_number = args.pop_front()
        .ok_or(String::from("Missing phone number!"))?;

    let response = backend_handles.bulkvs.lock()
        .await
        .query_phone_number(&phone_number)
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

    send_embed_no_return(
        ctx, 
        msg, 
        "CNAM Lookup", 
        &message, 
        get_random_anime_girl()
    ).await
        .unwrap();

    Ok(())
}
async fn geolocate(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
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

    send_embed_no_return(
        ctx, 
        msg, 
        "IP Lookup", 
        &message, 
        get_random_anime_girl()
    ).await
        .unwrap();

    Ok(())
}
pub async fn dehash(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
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
        send_embed_no_return(
            ctx.clone(), 
            msg.clone(), 
            "OSINT DUMP - `dehash`", 
            "There were more than 20 results, which in total contains more data than Discord can display.\n\nA full dump will be attached below shortly!", 
            get_random_anime_girl()
        ).await
            .unwrap();

        let builder = CreateMessage::new();

        tokio::spawn(msg.channel_id.send_files(
            ctx.http,
            std::iter::once(CreateAttachment::bytes(
                body.as_bytes(),
                "full_dump.txt"
            )),
            builder
        ));

        return Ok(());
    } else if total_results == 0 {
        send_embed_no_return(
            ctx, 
            msg, 
            "No results", 
            "There were no errors, but there were also no results!",
            get_random_anime_girl()
        ).await.expect("Failed to send message!");

        return Ok(());
    }

    send_embed_no_return(
        ctx, 
        msg, 
        "Dehash Results", 
        &body,
        get_random_anime_girl()
    ).await.expect("Failed to send message!");

    Ok(())
}
pub async fn rehash(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
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
        send_embed_no_return(
            ctx.clone(), 
            msg.clone(), 
            "OSINT DUMP - `rehash`", 
            "There were more than 20 results, which in total contains more data than Discord can display.\n\nA full dump will be attached below shortly!", 
            get_random_anime_girl()
        ).await
            .unwrap();

        let builder = CreateMessage::new();

        tokio::spawn(msg.channel_id.send_files(
            ctx.http,
            std::iter::once(CreateAttachment::bytes(
                body.as_bytes(),
                "full_dump.txt"
            )),
            builder
        ));

        return Ok(());
    } else if total_results == 0 {
        send_embed_no_return(
            ctx, 
            msg, 
            "No results", 
            "There were no errors, but there were also no results!",
            get_random_anime_girl()
        ).await.expect("Failed to send message!");

        return Ok(());
    }

    send_embed_no_return(
        ctx, 
        msg, 
        "Dehash Results", 
        &body,
        get_random_anime_girl()
    ).await.expect("Failed to send message!");

    Ok(())
}
pub async fn sherlock_helper(
    username: String,
    
    ctx: serenity::client::Context,
    msg: Message
) {
    let mut body = String::new();
    // Warn the user if the username is poor quality
    if !is_valid_sherlock_username(&username, false) {              
        body += &format!("### Warning\nThe username `{username}` has special characters and may not return quality results!");
    }

    let title = format!("OSINT - Sherlock - {username}");
    let url = get_random_anime_girl();
    let mut base_msg = send_embed(
            &ctx, 
            &msg, 
            &title, 
            "Preparing to search...", 
            &url
        ).await
            .unwrap();
    
    // Query Sherlock
    println!("Querying Sherlock for {username}");

    body += &format!("\n### {username}\n");

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

                body += &format!("{text}");            
                edit_embed(
                    &ctx,
                    &mut base_msg,
                    &title,
                    &body,
                    url
                ).await;
            }
        } else {
            break;
        }
    }

    if !found {
        body += &format!("\nNo results found for {username}");
        edit_embed(
            &ctx,
            &mut base_msg,
            &title,
            &body,
            &url
        ).await;
    }
}
pub async fn sherlock(
    _backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    mut args: VecDeque<String>
) -> Result<(), String> {
    let username = args.pop_front()
        .ok_or(String::from("Please provide a username!"))?;

    tokio::spawn(sherlock_helper(username, ctx, msg));

    Ok(())
}

pub async fn build_osint_commands() -> R6RSCommand {

    let mut osint_nest_command = R6RSCommand::new_root(
        String::from("Admin commands, generally intended only for usage by the owner.")
    );
    // Create a nest for query-based commands
    let mut query_nest_command = R6RSCommand::new_root(String::from("Query-based commands for OSINT."));
    
    // Attach the query-based commands to the query nest
    query_nest_command.attach(
        String::from("email"),
        R6RSCommand::new_leaf(
            String::from("Queries for leaks based on an email."),
            AsyncFnPtr::new(query_email),
            vec!(vec!(String::from("email")))
        )
    );
    query_nest_command.attach(
        String::from("username"),
        R6RSCommand::new_leaf(
            String::from("Queries for leaks based on a username."),
            AsyncFnPtr::new(query_username),
            vec!(vec!(String::from("username")))
        )
    );
    query_nest_command.attach(
        String::from("ip"),
        R6RSCommand::new_leaf(
            String::from("Queries for leaks based on a last IP."),
            AsyncFnPtr::new(query_last_ip),
            vec!(vec!(String::from("ip")))
        )
    );
    query_nest_command.attach(
        String::from("hash"),
        R6RSCommand::new_leaf(
            String::from("Queries for leaks based on a hash."),
            AsyncFnPtr::new(query_hash),
            vec!(vec!(String::from("hash")))
        )
    );
    query_nest_command.attach(
        String::from("password"),
        R6RSCommand::new_leaf(
            String::from("Queries for leaks based on a password."),
            AsyncFnPtr::new(query_password),
            vec!(vec!(String::from("password")))
        )
    );
    query_nest_command.attach(
        String::from("name"),
        R6RSCommand::new_leaf(
            String::from("Queries for leaks based on a name."),
            AsyncFnPtr::new(query_name),
            vec!(vec!(String::from("name")))
        )
    );

    // Finally, attach the query nest to the main nest
    osint_nest_command.attach(
        String::from("query"),
        query_nest_command
    );

    // Create the nest for hash-based commands
    let mut hash_nest_command = R6RSCommand::new_root(String::from("Hash-based commands for OSINT."));

    // Attach the hash-based commands to the hash nest
    hash_nest_command.attach(
        String::from("dehash"),
        R6RSCommand::new_leaf(
            String::from("Dehashes a hash into pre-cracked passwords."),
            AsyncFnPtr::new(dehash),
            vec!(vec!(String::from("hash")))
        )
    );
    hash_nest_command.attach(
        String::from("rehash"),
        R6RSCommand::new_leaf(
            String::from("Rehashes a password into pre-hashed hashes."),
            AsyncFnPtr::new(rehash),
            vec!(vec!(String::from("password")))
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
            vec!(vec!(String::from("phone number")))
        )
    );
    osint_nest_command.attach(
        String::from("geolocate"),
        R6RSCommand::new_leaf(
            String::from("Geolocates an IP."),
            AsyncFnPtr::new(geolocate),
            vec!(vec!(String::from("ip")))
        )
    );
    osint_nest_command.attach(
        String::from("sherlock"),
        R6RSCommand::new_leaf(
            String::from("Cross-references sites with a given username."),
            AsyncFnPtr::new(sherlock),
            vec!(vec!(String::from("username")))
        )
    );

    osint_nest_command
}
pub async fn osint ( 
    osint_nest_command: Arc<Mutex<R6RSCommand>>,

    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String> 
) {
    if let Err(err) = osint_nest_command.lock().await.call(
        backend_handles,
        ctx.clone(), 
        msg.clone(), 
        args
    ).await {
        println!("Failed! [{err}]");
        send_embed(
            &ctx, 
            &msg, 
            "OSINT - Blacklist Error", 
            &format!("Failed for reason:\n\n\"{err}\""), 
            get_random_anime_girl()
        ).await.unwrap();
    }
}