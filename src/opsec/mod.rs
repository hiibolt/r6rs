use crate::VecDeque;
use crate::Message;
use crate::Context;
use crate::send_embed;
use crate::helper::edit_embed;
use crate::UbisoftAPI;
use crate::Value;
use crate::{ Arc, Mutex };
use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead};


async fn get_profiles( ubisoft_api: Arc<Mutex<UbisoftAPI>>, account_id: &str ) -> Option<Vec<Value>> {
    let profiles: Value = ubisoft_api
        .lock().await
        .basic_request(format!("https://public-ubiservices.ubi.com/v3/users/{account_id}/profiles"))
        .await.ok()?;
    
    Some(profiles.get("profiles")?
        .as_array()?.clone())
}
async fn get_and_stringify_potential_profiles( 
    usernames: &Vec<String>, 
    ctx: &Context, 
    msg: &mut Message, 
    title: &str, 
    body: &mut String, 
    url: &str,
    no_special_characters: bool
) {
    let invalid_characters: [char; 4] = [' ', '.', '-', '_'];
    let invalid_sites: [&str; 20] = [
        "Oracle", "8tracks", "Coders Rank", "Fiverr",
        "HackerNews", "Modelhub", "metacritic", "xHamster",
        "CNET", "YandexMusic", "HackerEarth", "OpenStreetMap", 
        "Pinkbike", "Slides", "Strava", "Archive", "CGTrader",
        "G2G", "NationStates", "IFTTT"
    ];
    
    let valid_usernames: Vec<String> = usernames
        .iter()
        .filter(|username| {
            !no_special_characters || (!invalid_characters
                .iter()
                .any(|&ch| username.contains(ch)) 
                && 
            username
                .chars()
                .next().unwrap_or(' ')
                .is_alphabetic())
        })
        .map(|st| st.clone())
        .collect();

    // Query Sherlock
    for username in &valid_usernames {
        let mut cmd = Command::new("python")
            .arg("sherlock/sherlock")
            .arg("--nsfw")
            .arg("--folderoutput")
            .arg("sherlock_output")
            .arg(&format!("{username}"))
            .stdout(Stdio::piped())
            .spawn()
            .expect("Issue running the Sherlock command! Did you install with Nix?");
        {
            let stdout = cmd.stdout.as_mut().unwrap();
            let stdout_reader = BufReader::new(stdout);
            let stdout_lines = stdout_reader.lines();
    
            for line in stdout_lines {
                let output = line.unwrap_or(String::from(""));
                if invalid_sites
                        .iter()
                        .any(|site| output.contains(site))
                {
                    continue;
                }
                *body += &format!("\n{}", output);            
                edit_embed(
                    &ctx,
                    msg,
                    title,
                    &body,
                    url
                ).await;
            }
        }
        cmd.wait().unwrap();
    }
} 
fn stringify_profiles( profiles: &Vec<Value>, usernames: &mut Vec<String>, body: &mut String, account_id: &String ) {
    for profile in profiles {
        let username = profile["nameOnPlatform"]
            .as_str()
            .unwrap_or("");
        usernames.push(String::from(username));
        println!("Querying Sherlock for '{username}'...");
        match profile["platformType"].as_str() {
            Some("uplay") => {
                *body += &format!("### Uplay:\n- {} ({})\n- https://r6.tracker.network/r6/search?name={account_id}&platform=4\n",
                    username,
                    account_id
                    );
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
    ubisoft_api: Arc<Mutex<UbisoftAPI>>,
    ctx: Context,
    msg: Message,
    mut args: VecDeque<String>
) {
    let mut body = String::new();
    let title = "OPSEC - Uplay Linked Search";

    // Ensure input argument
    let input_option = args
        .pop_front();
    if input_option.is_none() {
        body += "Please supply an account ID or username!";

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
    let mut account_id = input_option
        .expect("Unreachable");

    // Ensure that input is an account ID
    match 
        ubisoft_api
            .lock().await
            .get_account_id(account_id.clone()).await
    {
        Some(id) => {
            account_id = String::from(id);
        }
        None => {
            body += &format!("Account **{account_id}** does not exist! Is it a PC ID?");

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
    let mut usernames: Vec<String> = Vec::new();
    
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
async fn namefind( 
    ctx: Context,
    msg: Message,
    args: VecDeque<String>
) { 
    let mut body = String::new();
    let title = "OPSEC - Namefind";

    // Ensure argument
    let usernames: Vec<String> = args
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
    ctx: Context,
    msg: Message
) {
    let _ = send_embed(
        &ctx, 
        &msg, 
        "OPSEC - Help", 
        "**Command list**:\n- `r6 opsec linked <pc uplay | account_id>`\n- `r6 opsec namefind <username1> <username2> ...`\n- `r6 opsec help`", 
        "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
    ).await
        .expect("Failed to send embed!");
}
pub async fn opsec( 
    ubisoft_api: Arc<Mutex<UbisoftAPI>>,
    ctx: Context,
    msg: Message,
    mut args: VecDeque<String> 
) {
    match args
        .pop_front()
        .unwrap_or(String::from("help"))
        .as_str()
    {
        "linked" => {
            tokio::spawn(linked( ubisoft_api, ctx, msg, args ));
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
                &format!("The command `r6 opsec {nonexistant}` is not valid!\n\nUse `r6 opsec help` for a complete command list."), 
                "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
            ).await
                .unwrap();
        }
    }
}