use crate::helper::get_random_anime_girl;
use crate::VecDeque;
use crate::Message;
use crate::send_embed;
use crate::helper::edit_embed;
use crate::Ubisoft;
use crate::Value;
use crate::{ Arc, Mutex };
use crate::env;
use anyhow::{ Result, anyhow };
use tokio::process::Command;
use std::collections::HashSet;
use std::process::Stdio;
use tokio::io::{BufReader, AsyncBufReadExt};


async fn get_profiles(
    ubisoft_api: Arc<Mutex<Ubisoft>>,
    account_id: &str
) -> Option<Vec<Value>> {
    let profiles: Value = ubisoft_api
        .lock().await
        .basic_request(format!("https://public-ubiservices.ubi.com/v3/users/{account_id}/profiles"))
        .await.ok()?;
    
    Some(profiles.get("profiles")?
        .as_array()?.clone())
}
async fn get_and_stringify_potential_profiles( 
    usernames: &HashSet<String>, 
    ctx: &serenity::client::Context, 
    msg: &mut Message, 
    title: &str, 
    body: &mut String, 
    url: &str,
    no_special_characters: bool
) {
    let invalid_characters: [char; 5] = [' ', '.', '-', '_', '#'];
    let invalid_sites: [&str; 38] = [
        "Oracle", "8tracks", "Coders Rank", "Fiverr",
        "HackerNews", "Modelhub", "metacritic", "xHamster",
        "CNET", "YandexMusic", "HackerEarth", "OpenStreetMap", 
        "Pinkbike", "Slides", "Strava", "Archive", "CGTrader",
        "G2G", "NationStates", "IFTTT", "SoylentNews", "hunting",
        "Contently", "Euw", "OurDJTalk", "BitCoinForum", "HEXRPG",
        "Polymart", "Linktree", "GeeksforGeeks", "Kongregate", "RedTube",
        "APClips", "Heavy-R", "RocketTube", "Zhihu", "NitroType", "babyRU"
    ];

    let mut invalid_usernames = HashSet::new();
    let mut valid_usernames = HashSet::new();

    for username in usernames.iter() {
        let allow_all: bool = !no_special_characters;
        let has_invalid_char: bool = !invalid_characters
            .iter()
            .any(|&ch| username.contains(ch));
        let has_alpha_first: bool = username
            .chars()
            .next().unwrap_or(' ')
            .is_alphabetic();
        let within_length: bool = username.chars().count() < 20;

        // If the username is bad, let the user know.
        if !(allow_all || ( has_invalid_char && has_alpha_first && within_length )) {
            invalid_usernames.insert(username.clone());

            continue;
        }

        valid_usernames.insert(username.clone());
    }

    // Query Sherlock
    for username in valid_usernames.iter() {
        println!("Querying Sherlock for {username}");
        let proxy_link = env::var("PROXY_LINK")
            .expect("Could not find PROXY_LINK in the environment!");
        
        println!("Starting with proxy link:\n\t{proxy_link}");
        
        let mut cmd = Command::new("python")
            .arg("sherlock/sherlock")
            .arg("--nsfw")
            .arg("--folderoutput")
            .arg("sherlock_output")
            .arg("--timeout")
            .arg("5")
            .arg("--proxy")
            .arg(proxy_link)
            .arg("--local")
            .arg(&format!("{username}"))
            .stdout(Stdio::piped())
            .spawn()
            .expect("Issue running the Sherlock command! Did you install with Nix?");
        {
            let stdout = cmd.stdout.as_mut().unwrap();
            let stdout_reader = BufReader::new(stdout);
            let mut stdout_lines = stdout_reader.lines();
    
            *body += &format!("\n### {username}\n");

            let mut found = false;
            while let Ok(Some(output)) = stdout_lines.next_line().await {
                if invalid_sites
                        .iter()
                        .any(|site| output.contains(site))
                {
                    continue;
                }

                if output.contains("http") || output.contains("https") {
                    println!("Found site for {username}: {output}");

                    found = true;

                    *body += &format!("\n{output}");            
                    edit_embed(
                        &ctx,
                        msg,
                        title,
                        &body,
                        url
                    ).await;
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
        cmd.wait().await.unwrap();
    }
    
    if invalid_usernames.len() > 0 {
        *body += "\n### Ignored Usernames\n";
        
        *body += &invalid_usernames.into_iter()
            .map(|username| format!("- {username}"))
            .collect::<Vec<String>>()
            .join("\n");

        *body += "\n\nThese usernames would produce poor results from Sherlock. You can always run them manually with the following command :)\n`>>r6 opsec namefind <username>`";

        edit_embed(
            &ctx,
            msg,
            title,
            &body,
            url
        ).await;
    }
} 
fn stringify_profiles(
    profiles: &Vec<Value>,
    usernames: &mut HashSet<String>,
    body: &mut String,
    account_id: &String
) {
    for profile in profiles {
        let username = profile["nameOnPlatform"]
            .as_str()
            .unwrap_or("");
        usernames.insert(String::from(username));
        match profile["platformType"].as_str() {
            Some("uplay") => {
                *body += &format!("### Uplay:\n- {username} ({account_id})\n- https://stats.cc/siege/{username}/{account_id}/playedWith\n- https://r6.tracker.network/r6/search?name={account_id}&platform=4\n");
            },
            Some("steam") => {
                *body += &format!("**Steam**:\n- https://findsteamid.com/steamid/{}\n- https://steamid.io/lookup/{}\n", profile["idOnPlatform"].as_str().unwrap_or(""), profile["idOnPlatform"].as_str().unwrap_or(""));
            },
            Some("xbl") => {
                let with_htmlsafe = profile["nameOnPlatform"]
                    .as_str()
                    .unwrap_or("")
                    .replace(" ", "%20");
                let with_pluses = profile["nameOnPlatform"]
                    .as_str()
                    .unwrap_or("")
                    .replace(" ", "%20");
                *body += &format!("**XBL**:\n- {} ({})\n- https://r6.tracker.network/r6/search?name={}&platform=1\n- https://xboxgamertag.com/search/{}\n",
                        profile["nameOnPlatform"]
                        .as_str()
                        .unwrap_or(""),
                        profile["idOnPlatform"]
                        .as_str()
                        .unwrap_or(""),
                        with_pluses,
                        with_htmlsafe
                    );
            },
            Some("psn") => {
                let id = profile["idOnPlatform"]
                    .as_str()
                    .unwrap_or("");
                *body += &format!("**PSN**:\n- {} ({})\n- https://r6.tracker.network/r6/search?name={}&platform=2\n- https://psnprofiles.com/search/users?q={}\n",
                        username,
                        id,
                        username,
                        username
                    );
            },
            Some("twitch") => {
                let id = profile["idOnPlatform"]
                    .as_str()
                    .unwrap_or("");
                *body += &format!("**Twitch**:\n- {} ({})\n- https://www.twitch.tv/{}\n",
                    username,
                    id,
                    username
                );
            },
            Some(_) => {
                let platform = profile["platformType"]
                    .as_str()
                    .unwrap_or("");
                let id = profile["idOnPlatform"]
                    .as_str()
                    .unwrap_or("");
                *body += &format!("**{platform}**:\n- {username}\n- {id}\n");
            }
            None => {
                *body += "Could not find any linked platforms!\n";
            }
        }
    }
}
async fn linked(
    ubisoft_api: Arc<Mutex<Ubisoft>>,
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String>,
    platform: String
) {
    let mut body = String::new();
    let title = "OPSEC - Uplay Linked Search";

    // Ensure input argument
    let mut account_id = args
        .into_iter()
        .collect::<Vec<String>>()
        .join(" ");
    if account_id == "" {
        body += "Please supply an account ID or username!";

        send_embed(
            &ctx, 
            &msg, 
            title, 
            &body, 
            get_random_anime_girl()
        ).await
            .unwrap();

        return;
    }

    // Ensure that input is an account ID
    match 
        ubisoft_api
            .lock().await
            .get_account_id(account_id.clone(), platform).await
    {
        Ok(id) => {
            account_id = String::from(id);
        }
        Err(_) => {
            body += &format!("Account **{account_id}** does not exist!");

            send_embed(
                &ctx, 
                &msg, 
                title, 
                &body, 
                "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
            ).await
                .unwrap();

            return;
        }
    }
    
    // Ensure valid account ID
    let profiles_option: Option<Vec<Value>> = get_profiles( ubisoft_api.clone(), &account_id )
        .await;
    if profiles_option.is_none() {
        let _ = send_embed(
            &ctx, 
            &msg, 
            title, 
            "Account ID does not exist!", 
            "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
        ).await
            .unwrap();
        return;
    }
    let profiles = profiles_option
        .expect("Unreachable");
    let mut usernames: HashSet<String> = HashSet::new();
    
    body += "## ⛓️ Linked Profiles\n";
    stringify_profiles( &profiles, &mut usernames, &mut body, &account_id );

    let mut sent = send_embed(
        &ctx, 
        &msg, 
        title, 
        &body, 
        &format!("https://ubisoft-avatars.akamaized.net/{account_id}/default_tall.png")
    ).await
        .unwrap();

    body += "## ❔ Potential Profiles\n";
    get_and_stringify_potential_profiles (
        &usernames,
        &ctx,
        &mut sent,
        title,
        &mut body,
        &format!("https://ubisoft-avatars.akamaized.net/{account_id}/default_tall.png"),
        true
    ).await;
}
async fn applications(
    ubisoft_api: Arc<Mutex<Ubisoft>>,
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String>
) {
    let mut body = String::new();
    let title = "OPSEC - Applications";

    if args.len() == 0 {
        body += "Please supply an account ID or username!";

        send_embed(
            &ctx, 
            &msg, 
            title, 
            &body, 
            get_random_anime_girl()
        ).await
            .unwrap();

        return;
    }

    // Ensure input argument
    let mut account_id = args
        .into_iter()
        .collect::<Vec<String>>()
        .join(" ");
    if account_id == "" {
        body += "Please supply an account ID or username!";

        send_embed(
            &ctx, 
            &msg, 
            title, 
            &body, 
            get_random_anime_girl()
        ).await
            .unwrap();

        return;
    }

    // Ensure that input is an account ID
    match 
        ubisoft_api
            .lock().await
            .get_account_id(account_id.clone(), String::from("uplay")).await
    {
        Ok(id) => {
            account_id = String::from(id);
        }
        Err(_) => {
            body += &format!("Account **{account_id}** does not exist!");

            send_embed(
                &ctx, 
                &msg, 
                title, 
                &body, 
                "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
            ).await
                .unwrap();

            return;
        }
    }

    let res = ubisoft_api.lock().await
        .get_applications(account_id.clone()).await
        .expect("Failed to get applications!");

    match serialize_applications_response( &res ) {
        Ok(applications) => {
            body += &format!("## 📱 Applications\n\n");

            body += &applications;
        }
        Err(err) => {
            body += &format!("\nEncountered an error while fetching applications! \n{err}");

            send_embed(
                &ctx, 
                &msg, 
                title, 
                &body, 
                "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
            ).await
                .unwrap();
        }
    }

    send_embed(
        &ctx, 
        &msg, 
        title, 
        &body, 
        &format!("https://ubisoft-avatars.akamaized.net/{account_id}/default_tall.png")
    ).await
        .unwrap();

    println!("Result: {res}");
}
fn serialize_applications_response (
    res: &Value
) -> Result<String> {
    let mut body = String::new();

    let applications = res.get("applications")
        .ok_or(anyhow!("Failed to get applications!"))?
        .as_array()
        .ok_or(anyhow!("Failed to get applications!"))?;

    for application_value in applications {
        let application = application_value
            .as_object()
            .ok_or(anyhow!("Failed to get application!"))?;

        let app_id = application.get("appId")
            .and_then(|val| val.as_str())
            .unwrap_or("Unknown");

        body += &format!("### {app_id}\n");

        for (key, value) in application {
            if key == "appId" {
                continue;
            }
            body += &format!("**{key}**: {value}\n");
        }
    }

    Ok(body)
}
async fn namefind( 
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String>
) { 
    let mut body = String::new();
    let title = "OPSEC - Namefind";

    // Ensure argument
    let usernames: HashSet<String> = args
        .into_iter()
        .collect();
    if usernames.len() == 0 {
        body += "Please supply a username!";

        send_embed(
            &ctx, 
            &msg, 
            title, 
            &body, 
            "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
        ).await
            .unwrap();

        return;
    }
    
    body += &format!("Searching for {:?}", usernames);
    
    let mut sent_msg = send_embed(
        &ctx, 
        &msg, 
        title, 
        &body, 
        "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
    ).await
        .expect("Failed to send embed!");

    get_and_stringify_potential_profiles(
        &usernames, 
        &ctx, 
        &mut sent_msg, 
        title, 
        &mut body, 
        "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4",
        false
    ).await;   
}
async fn help(
    ctx: serenity::client::Context,
    msg: Message
) {
    let _ = send_embed(
        &ctx, 
        &msg, 
        "R6 - OPSEC - Help", 
        "**Command List**:\n- `>>r6 opsec <pc | xbox | psn> <account name>`\n- `>>r6 opsec namefind <username1> <username2> ...`\n-`>>r6 opsec applications <pc uplay>`\n- `>>r6 opsec help`", 
        "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
    ).await
        .expect("Failed to send embed!");
}
pub async fn opsec( 
    ubisoft_api: Arc<Mutex<Ubisoft>>,
    ctx: serenity::client::Context,
    msg: Message,
    mut args: VecDeque<String> 
) {
    match args
        .pop_front()
        .unwrap_or(String::from("help"))
        .as_str()
    {
        "pc" => {
            tokio::spawn(linked( ubisoft_api, ctx, msg, args, String::from("uplay") ));
        },
        "xbox" => {
            tokio::spawn(linked( ubisoft_api, ctx, msg, args, String::from("xbl") ));
        },
        "psn" => {
            tokio::spawn(linked( ubisoft_api, ctx, msg, args, String::from("psn") ));
        },
        "applications" => {
            tokio::spawn(applications( ubisoft_api, ctx, msg, args ));
        }, 
        "namefind" => {
            tokio::spawn(namefind( ctx, msg, args ));
        },
        "help" => {
            tokio::spawn(help( ctx, msg ));
        },
        nonexistant => {
            send_embed(
                &ctx, 
                &msg, 
                "Command does not exist", 
                &format!("The subcommand `{nonexistant}` is not valid!\n\nConfused?\nRun `>>r6 opsec help` for information on `opsec`'s commands\nRun `r6 help` for information on all commands"), 
                "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
            ).await
                .unwrap();
        }
    }
}
