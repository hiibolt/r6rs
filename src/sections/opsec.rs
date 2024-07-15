use crate::apis::get_and_stringify_potential_profiles;
use crate::helper::get_random_anime_girl;
use crate::helper::send_embed_no_return;
use crate::helper::AsyncFnPtr;
use crate::helper::BackendHandles;
use crate::helper::R6RSCommand;
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
) -> Result<(), String> {
    let mut body = String::new();
    let title = "OPSEC - Uplay Linked Search";

    // Ensure input argument
    let mut account_id = args
        .into_iter()
        .collect::<Vec<String>>()
        .join(" ");
    if account_id == "" {
        return Err(String::from("Please supply an account ID or username!"));
    }

    // Ensure that input is an account ID
    account_id = ubisoft_api
        .lock().await
        .get_account_id(account_id.clone(), platform).await
        .map_err(|_| format!("Account **{account_id}** does not exist!"))?;
    
    // Ensure valid account ID
    let profiles: Vec<Value> = get_profiles( ubisoft_api.clone(), &account_id )
        .await
        .ok_or(format!("Account ID **{account_id}** does not exist!"))?;
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
        false
    ).await;

    Ok(())
}
async fn applications_helper(
    ubisoft_api: Arc<Mutex<Ubisoft>>,
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String>
) -> Result<(), String> {
    let mut body = String::new();
    let title = "OPSEC - Applications";

    if args.len() == 0 {
        return Err(String::from("Please supply an account ID or username!"));
    }

    // Ensure input argument
    let mut account_id = args
        .into_iter()
        .collect::<Vec<String>>()
        .join(" ");
    if account_id == "" {
        return Err(String::from("Please supply an account ID or username!"));
    }

    // Ensure that input is an account ID
    account_id =  ubisoft_api
        .lock().await
        .get_account_id(account_id.clone(), String::from("uplay")).await
        .map_err(|_| format!("Account **{account_id}** does not exist!"))?;

    let res = ubisoft_api.lock().await
        .get_applications(account_id.clone()).await
        .expect("Failed to get applications!");

    let applications = serialize_applications_response( &res )
        .map_err(|err| format!("\nEncountered an error while fetching applications! \n{err}"))?;
    
    body += &format!("## 📱 Applications\n\n");
    body += &applications;

    send_embed_no_return(
        ctx, 
        msg, 
        title, 
        &body, 
        &format!("https://ubisoft-avatars.akamaized.net/{account_id}/default_tall.png")
    ).await
        .unwrap();

    println!("Result: {res}");

    Ok(())
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
pub async fn lookup_pc(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String>
) -> Result<(), String> {
    tokio::spawn(linked( backend_handles.ubisoft_api, ctx, msg, args, String::from("uplay")));

    Ok(())
}
pub async fn lookup_xbox(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String>
) -> Result<(), String> {
    tokio::spawn(linked( backend_handles.ubisoft_api, ctx, msg, args, String::from("xbl")));

    Ok(())
}
pub async fn lookup_psn(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String>
) -> Result<(), String> {
    tokio::spawn(linked( backend_handles.ubisoft_api, ctx, msg, args, String::from("psn")));

    Ok(())
}
pub async fn applications(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String>
) -> Result<(), String> {
    applications_helper( backend_handles.ubisoft_api, ctx, msg, args ).await
}

pub async fn build_opsec_commands() -> R6RSCommand {
    let mut opsec_nest_command = R6RSCommand::new_root(
        String::from("Commands for location information on Ubisoft Connect accounts.")
    );
    opsec_nest_command.attach(
        String::from("pc"),
        R6RSCommand::new_leaf(
            String::from("Lookups up a Ubisoft account based on their registered PC username"),
            AsyncFnPtr::new(lookup_pc),
            vec!(vec!(String::from("username")))
        )
    );
    opsec_nest_command.attach(
        String::from("xbox"),
        R6RSCommand::new_leaf(
            String::from("Lookups up a Ubisoft account based on their registered Xbox username"),
            AsyncFnPtr::new(lookup_xbox),
            vec!(vec!(String::from("username")))
        )
    );
    opsec_nest_command.attach(
        String::from("psn"),
        R6RSCommand::new_leaf(
            String::from("Lookups up a Ubisoft account based on their registered PSN username"),
            AsyncFnPtr::new(lookup_psn),
            vec!(vec!(String::from("username")))
        )
    );
    opsec_nest_command.attach(
        String::from("applications"),
        R6RSCommand::new_leaf(
            String::from("Lookups up a Ubisoft account based on their username (PC only)"),
            AsyncFnPtr::new(applications),
            vec!(vec!(String::from("username")))
        )
    );

    opsec_nest_command
}
pub async fn opsec( 
    opsec_nest_command: Arc<Mutex<R6RSCommand>>,

    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: Message,
    args: VecDeque<String> 
) {
    if let Err(err) = opsec_nest_command.lock().await.call(
        backend_handles,
        ctx.clone(), 
        msg.clone(), 
        args
    ).await {
        println!("Failed! [{err}]");
        send_embed(
            &ctx, 
            &msg, 
            "OPSEC - Blacklist Error", 
            &format!("Failed for reason:\n\n\"{err}\""), 
            get_random_anime_girl()
        ).await.unwrap();
    }
}
