use crate::apis::get_and_stringify_potential_profiles;
use crate::helper::get_random_anime_girl;
use crate::VecDeque;
use crate::Message;
use crate::send_embed;
use crate::Ubisoft;
use crate::Value;
use crate::{ Arc, Mutex };
use anyhow::{ Result, anyhow };
use std::collections::HashSet;


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
    let application_ids = std::collections::HashMap::from([
        ("f68a4bb5-608a-4ff2-8123-be8ef797e0a6", "Rainbow Six Siege - PC (Ubisoft Connect)"),
        ("e3d5ea9e-50bd-43b7-88bf-39794f4e3d40", "Rainbow Six Siege - TTS (Ubisoft Connect)")
    ]);
    
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

        if let Some(app_name) = application_ids.get(app_id) {
            body += &format!("### {app_name}\n");
        } else {
            body += &format!("### Unknown ({app_id})\n");
        }

        for (key, value) in application {
            if key == "appId" {
                continue;
            }
            body += &format!("**{key}**: {value}\n");
        }
    }

    Ok(body)
}
async fn help(
    ctx: serenity::client::Context,
    msg: Message
) {
    let _ = send_embed(
        &ctx, 
        &msg, 
        "R6 - OPSEC - Help", 
        "**Command List**:\n- `>>r6 opsec <pc | xbox | psn> <account name>`\n- `>>r6 opsec applications <pc uplay>`\n- `>>r6 opsec help`", 
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
            send_embed(
                &ctx, 
                &msg, 
                "Command depreciated", 
                "Please instead use the following:\n`>>osint sherlock <username>`", 
                "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
            ).await
                .unwrap();
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
