use crate::apis::{ snusbase::Snusbase, bulkvs::BulkVS };
use crate::helper::{ get_random_anime_girl, send_embed };

use serenity::all::{CreateAttachment, CreateMessage, Message};
use tokio::sync::Mutex;
use std::{collections::VecDeque, sync::Arc};



pub async fn lookup( 
    snusbase: Arc<Mutex<Snusbase>>,
    ctx: serenity::client::Context,
    msg: Message,
    mut args: VecDeque<String>,
    lookup_type: &str
) {
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
        send_embed(
            &ctx, 
            &msg, 
            "An error occured", 
            &format!("{}", snusbase_response_result.unwrap_err()), 
            get_random_anime_girl()
        ).await
            .unwrap();

        return;
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

        send_embed(
            &ctx, 
            &msg, 
            "OSINT DUMP", 
            "There were more than 10 results, which in total contains more data than Discord can display.\n\nA full dump will be attached below shortly!", 
            get_random_anime_girl()
        ).await
            .unwrap();

        let builder = CreateMessage::new();

        msg.channel_id.send_files(
            &ctx.http,
            std::iter::once(CreateAttachment::bytes(
                full_dump.as_bytes(),
                "full_dump.txt"
            )),
            builder
        ).await
            .unwrap();

        return;
    }

    if snusbase_response.results.len() == 0 {
        send_embed(
            &ctx, 
            &msg, 
            "No results", 
            "Nothing was found for the given query!\n\n*There were no errors, but there weren't any results either.*", 
            get_random_anime_girl()
            ).await
                .unwrap();
        
        return;
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

            send_embed(
                &ctx, 
                &msg, 
                "OSINT DUMP - Via Email", 
                &message, 
                get_random_anime_girl()
            ).await
                .unwrap();
        }
    }
}
pub async fn help (
    ctx: serenity::client::Context,
    msg: Message
) {
    send_embed(
        &ctx, 
        &msg, 
        "OSINT Help", 
        concat!("The `osint` command is used to query for information on emails, usernames, IPs, passwords and names.\n\n",
        "## Subcommands:\n",
        "### Personal Information\n",
        "- `email` - Query database leaks by email\n", 
        "- `username` - Query database leaks by username\n", 
        "- `name` - Query database leaks by name\n",
        "### Passwords\n",
        "- `password` - Query database leaks by password\n",
        "- `hash` - Query database leaks by hash\n",
        "- `dehash` - Dehash a hash into pre-cracked passwords\n",
        "- `rehash` - Rehash a password into pre-hashed hashes\n",
        "### IP and Cellular\n",
        "- `ip` - Geolocate by IP\n",
        "- `last_ip` Query database leaks by IP\n",
        "- `phone` - Perform CNAM lookup\n\n",
        "**Usage**:\n",
        "- `>>osint email <email>`\n",
        "- `>>osint username <username>`\n",
        "- `>>osint password <password>`\n",
        "- `>>osint name <name>`\n",
        "- `>>osint hash <hash>`\n",
        "- `>>osint dehash <hash>`\n",
        "- `>>osint rehash <password>`\n",
        "- `>>osint ip <ip>`\n",
        "- `>>osint last_ip <last ip>`\n",
        "- `>>osint phone <phone number>`"), 
        get_random_anime_girl()
            ).await
                .unwrap();
}
pub async fn osint ( 
    snusbase: Arc<Mutex<Snusbase>>,
    bulkvs: Arc<Mutex<BulkVS>>,
    ctx: serenity::client::Context,
    msg: Message,
    mut args: VecDeque<String> 
) {
    match args
        .pop_front()
        .unwrap_or(String::from("help"))
        .as_str()
    {
        "email" => {
            tokio::spawn(lookup(snusbase, ctx, msg, args, "email"));
        },
        "username" => {
            tokio::spawn(lookup(snusbase, ctx, msg, args, "username"));
        },
        "last_ip" => {
            tokio::spawn(lookup(snusbase, ctx, msg, args, "last_ip"));
        },
        "hash" => {
            tokio::spawn(lookup(snusbase, ctx, msg, args, "hash"));
        },
        "password" => {
            tokio::spawn(lookup(snusbase, ctx, msg, args, "password"));
        },
        "name" => {
            tokio::spawn(lookup(snusbase, ctx, msg, args, "name"));
        },
        "phone" => {
            if let Some(phone_number) = args.pop_front() {
                let response = bulkvs.lock()
                    .await
                    .query_phone_number(&phone_number);

                if response.is_err() {
                    send_embed(
                        &ctx, 
                        &msg, 
                        "An error occured", 
                        &format!("{}", response.unwrap_err()), 
                        get_random_anime_girl()
                    ).await
                        .unwrap();
    
                    return;
                }

                let response = response.unwrap();
    
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

                send_embed(
                    &ctx, 
                    &msg, 
                    "CNAM Lookup", 
                    &message, 
                    get_random_anime_girl()
                ).await
                    .unwrap();

                return;
            }
            
            send_embed(
                &ctx, 
                &msg, 
                "An error occured", 
                "Missing phone number!", 
                get_random_anime_girl()
            ).await
                .unwrap();
        },
        "ip" => {
            let response = snusbase.lock()
                .await
                .whois_ip_query(args.into_iter().collect())
                .await;

            if response.is_err() {
                send_embed(
                    &ctx, 
                    &msg, 
                    "An error occured", 
                    &format!("{}", response.unwrap_err()), 
                    get_random_anime_girl()
                ).await
                    .unwrap();

                return;
            }

            let response = response.unwrap();

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

            send_embed(
                &ctx, 
                &msg, 
                "IP Lookup", 
                &message, 
                get_random_anime_girl()
            ).await
                .unwrap();
        },
        "dehash" => {
            let response = snusbase.lock()
                .await
                .dehash(args.into_iter().collect())
                .await;

            if let Err(e) = response {
                let error_as_string = format!("{}", e);
                if &error_as_string == "Failed to convert response to string!" {
                    send_embed(
                        &ctx, 
                        &msg, 
                        "An error occured", 
                        &format!("Couldn't deserialize response! This probably means there were way, way too many results.\n\nDon't ask for things like `password123`!\n\nRaw Error: `{}`", error_as_string),
                        get_random_anime_girl()
                    ).await.expect("Failed to send message!");
    
                    return;

                }

                send_embed(
                    &ctx, 
                    &msg, 
                    "An error occured", 
                    &format!("{}", error_as_string),
                    get_random_anime_girl()
                ).await.expect("Failed to send message!");

                return;
            }

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
                send_embed(
                    &ctx, 
                    &msg, 
                    "OSINT DUMP - `dehash`", 
                    "There were more than 20 results, which in total contains more data than Discord can display.\n\nA full dump will be attached below shortly!", 
                    get_random_anime_girl()
                ).await
                    .unwrap();
        
                let builder = CreateMessage::new();
        
                msg.channel_id.send_files(
                    &ctx.http,
                    std::iter::once(CreateAttachment::bytes(
                        body.as_bytes(),
                        "full_dump.txt"
                    )),
                    builder
                ).await
                    .unwrap();

                return;
            } else if total_results == 0 {
                send_embed(
                    &ctx, 
                    &msg, 
                    "No results", 
                    "No results were found for the given query!",
                    get_random_anime_girl()
                ).await.expect("Failed to send message!");

                return;
            }

            send_embed(
                &ctx, 
                &msg, 
                "Dehash Results", 
                &body,
                get_random_anime_girl()
            ).await.expect("Failed to send message!");
        },
        "rehash" => {
            let response = snusbase.lock()
                .await
                .rehash(args.into_iter().collect())
                .await;

            if let Err(e) = response {
                let error_as_string = format!("{}", e);
                if &error_as_string == "Failed to convert response to string!" {
                    send_embed(
                        &ctx, 
                        &msg, 
                        "An error occured", 
                        &format!("Couldn't deserialize response! This probably means there were way, way too many results.\n\nDon't ask for things like `password123`!\n\nRaw Error: `{}`", error_as_string),
                        get_random_anime_girl()
                    ).await.expect("Failed to send message!");
    
                    return;

                }

                send_embed(
                    &ctx, 
                    &msg, 
                    "An error occured", 
                    &format!("{}", error_as_string),
                    get_random_anime_girl()
                ).await.expect("Failed to send message!");

                return;
            }

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
                send_embed(
                    &ctx, 
                    &msg, 
                    "OSINT DUMP - `rehash`", 
                    "There were more than 20 results, which in total contains more data than Discord can display.\n\nA full dump will be attached below shortly!", 
                    get_random_anime_girl()
                ).await
                    .unwrap();
        
                let builder = CreateMessage::new();
        
                msg.channel_id.send_files(
                    &ctx.http,
                    std::iter::once(CreateAttachment::bytes(
                        body.as_bytes(),
                        "full_dump.txt"
                    )),
                    builder
                ).await
                    .unwrap();

                return;
            } else if total_results == 0 {
                send_embed(
                    &ctx, 
                    &msg, 
                    "No results", 
                    "No results were found for the given query!",
                    get_random_anime_girl()
                ).await.expect("Failed to send message!");

                return;
            }

            send_embed(
                &ctx, 
                &msg, 
                "Dehash Results", 
                &body,
                get_random_anime_girl()
            ).await.expect("Failed to send message!");
        }
        "help" => {
            tokio::spawn(help( ctx, msg ));
        },
        nonexistant => {
            send_embed(
                &ctx, 
                &msg, 
                "Command does not exist", 
                &format!("The subcommand `{nonexistant}` is not valid!\n\nConfused?\nRun `>>osint help` for information on `osint`'s commands\nRun `r6 help` for information on all commands"), 
                get_random_anime_girl()
            ).await
                .unwrap();
        }
    }
}