use crate::VecDeque;
use crate::Message;
use crate::Context;
use crate::unimplemented;
use crate::send_embed;
use crate::UbisoftAPI;
use crate::State;
use crate::Value;
use crate::{ Arc, Mutex };
use tungstenite::{connect};
use url::Url;

async fn get_profiles( ubisoft_api: Arc<Mutex<UbisoftAPI>>, account_id: &str ) -> Option<Vec<Value>> {
    let profiles: Value = ubisoft_api
        .lock().await
        .basic_request(format!("https://public-ubiservices.ubi.com/v3/users/{account_id}/profiles"))
        .await.ok()?;
    
    println!("{:?}", profiles.get("profiles"));
    
    Some(profiles.get("profiles")?
        .as_array()?.clone())
}
async fn get_and_stringify_potential_profiles( usernames: &Vec<String>, body: &mut String ) {
    let invalid_characters: [char; 4] = [' ', '.', '-', '_'];
    
    let valid_usernames: Vec<String> = usernames
        .iter()
        .filter(|username| {
            !invalid_characters
                .iter()
                .any(|&ch| username.contains(ch)) 
                && 
            username
                .chars()
                .next().unwrap_or(' ')
                .is_alphabetic()
        })
        .map(|st| st.clone())
        .collect();

    let (mut socket, _) = connect(Url::parse("wss://echo.websocket.in").unwrap()).expect("Can't connect");

    println!("Connected to the server");
    loop {
        let msg = socket.read().expect("Error reading message");
        println!("Received: {}", msg);
    }
    // socket.close(None);
    
    
    println!("{:?}", valid_usernames)

} 
fn stringify_profiles( profiles: &Vec<Value>, usernames: &mut Vec<String>, body: &mut String, account_id: &String ) {
    for profile in profiles {
        let username = profile["nameOnPlatform"]
            .as_str()
            .unwrap_or("");
        usernames.push(String::from(username));
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
    let title = "OPSEC - Full Data Dump";

    // Ensure input argument
    let input_option = args
        .pop_front();
    if input_option.is_none() {
        body += "Please supply an account ID or username!";

        send_embed(
            ctx, 
            msg, 
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
                ctx, 
                msg, 
                title, 
                &body, 
                "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
            ).await
                .unwrap();

            return;
        }
    }
    
    let profiles: Vec<Value> = get_profiles( ubisoft_api.clone(), &account_id )
        .await
        .unwrap_or(Vec::new());
    let mut usernames: Vec<String> = Vec::new();
    
    body += "## ⛓️ Linked Profiles\n";
    stringify_profiles( &profiles, &mut usernames, &mut body, &account_id );

    body += "## ❔ Potential Profiles\n";
    get_and_stringify_potential_profiles ( &usernames, &mut body ).await;


    send_embed(
        ctx, 
        msg, 
        title, 
        &body, 
        &format!("https://ubisoft-avatars.akamaized.net/{account_id}/default_tall.png")
    ).await
        .unwrap();
}
async fn dump( 
    ubisoft_api: Arc<Mutex<UbisoftAPI>>,
    ctx: Context,
    msg: Message,
    mut args: VecDeque<String>
) { 
    let mut body = String::new();
    let title = "OPSEC - Full Data Dump";

    // Ensure argument
    let input_option = args
        .pop_front();
    if input_option.is_none() {
        body += "Please supply an account ID or username!";

        send_embed(
            ctx, 
            msg, 
            title, 
            &body, 
            "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
        ).await
            .unwrap();

        return;
    }
    let mut account_id = input_option
        .expect("Unreachable");

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
                ctx, 
                msg, 
                title, 
                &body, 
                "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
            ).await
                .unwrap();

            return;
        }
    }
    
    let player: Option<Value> = ubisoft_api
        .lock().await
        .basic_request(format!("https://public-ubiservices.ubi.com/v1/profiles/{account_id}"))
        .await.ok();
    let persona: Option<Value> = ubisoft_api
        .lock().await
        .basic_request(format!("https://public-ubiservices.ubi.com/v1/profiles/persona?profileIds={account_id}&spaceId=0d2ae42d-4c27-4cb7-af6c-2099062302bb"))
        .await.ok();
    let stats: Option<Value> = ubisoft_api
        .lock().await
        .basic_request(format!("https://public-ubiservices.ubi.com/v2/spaces/0d2ae42d-4c27-4cb7-af6c-2099062302bb/title/r6s/skill/full_profiles?profile_ids={}&platform_families=pc", account_id))
        .await.ok();
    let profiles: Vec<Value> = get_profiles( ubisoft_api.clone(), &account_id )
        .await
        .unwrap_or(Vec::new());

    println!("{account_id} - {profiles:?}");

    body += &account_id;

    send_embed(
        ctx, 
        msg, 
        title, 
        &body, 
        "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
    ).await
        .unwrap();
}
pub async fn opsec( 
    ubisoft_api: Arc<Mutex<UbisoftAPI>>,
    state: Arc<Mutex<State>>,
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
            linked( ubisoft_api, ctx, msg, args ).await;
        },
        "dump" => {
            dump( ubisoft_api, ctx, msg, args ).await;
        },
        "help" => {
            unimplemented( ctx, msg, "help" ).await;
        },
        nonexistant => {
            send_embed(
                ctx, 
                msg, 
                "Command does not exist", 
                &format!("The command **{nonexistant}** is not valid!"), 
                "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
            ).await
                .unwrap();
        }
    }
}