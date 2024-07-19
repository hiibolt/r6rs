use crate::{
    apis::get_and_stringify_potential_profiles, helper::{
        bot::{BackendHandles, GenericMessage}, command::R6RSCommand, lib::{get_random_anime_girl, send_embed_no_return, AsyncFnPtr}
    }, info, send_embed, startup, warn, Arc, Colorize, Mutex, Ubisoft, Value, VecDeque
};
use regex::Regex;
use scraper::{Html, Selector};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use anyhow::{Result, anyhow};

#[derive(Debug)]
pub struct PlayedWithPlayer<'a> {
    pub _name: &'a str,
    pub _last_played: &'a str,
    pub _kd: f64,
    pub _wr: f64,
    pub games_played: u32,
    pub rep_ban: bool,
    pub hard_ban: bool,
    pub rank: Option<u32>
}
impl<'a> PlayedWithPlayer<'a> {
    pub fn from(
        args: Vec<&'a str>,
        rep_ban: bool,
        hard_ban: bool,
        rank: Option<u32>
    ) -> Option<Self> {
        let kd: f64 = args.get(2)
            .and_then(|st| st.replace(" KD", "").parse().ok())?;
        let wr: f64 = args.get(3)
            .and_then(|st| st.replace("% WR", "").parse().ok())?;
        let games_played: u32 = args.get(4)
            .and_then(|st| st.parse().ok())?;

        Some(Self {
            _name: args.get(0)?,
            _last_played: args.get(1)?,
            _kd: kd,
            _wr: wr,
            games_played,
            rep_ban,
            hard_ban,
            rank
        })
    }
}

#[derive(Debug, Deserialize)]
struct FindSteamIDBanInfo {
    communitybanned: bool,
    vacbanned: bool,
    numberofvacbans: usize,
    economyban: String
}
#[derive(Deserialize)]
pub struct GPTResponse {
    cheating: bool,
    concrete_evidence: Vec<GPTFlagEntry>
}
#[derive(Deserialize)]
pub struct GPTFlagEntry {
    line_numbers: String,
    cheat_type: String,
    confidence: String,
    reasoning: String
}

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
async fn linked_helper(
    ubisoft_api: Arc<Mutex<Ubisoft>>,
    ctx: serenity::client::Context,
    msg: GenericMessage,
    args: VecDeque<String>,
    platform: String,
    use_sherlock: bool
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
        &msg.channel_id, 
        title, 
        &body, 
        &format!("https://ubisoft-avatars.akamaized.net/{account_id}/default_tall.png")
    ).await
        .unwrap();

    if use_sherlock {
        body += "## ❔ Potential Profiles\n";
        if let Err(e) = get_and_stringify_potential_profiles (
            &usernames,
            &ctx,
            &mut sent,
            title,
            &mut body,
            &format!("https://ubisoft-avatars.akamaized.net/{account_id}/default_tall.png"),
            false
        ).await {
            body += &format!("Failed to get potential profiles!\n\n{e:#?}");

            warn!("Failed to get potential profiles!\n\n{e:#?}");

            send_embed_no_return(
                ctx, 
                msg.channel_id, 
                title, 
                &body, 
                &format!("https://ubisoft-avatars.akamaized.net/{account_id}/default_tall.png")
            ).await
                .unwrap();
        }
    }

    Ok(())
}
async fn linked(
    ubisoft_api: Arc<Mutex<Ubisoft>>,
    ctx: serenity::client::Context,
    msg: GenericMessage,
    args: VecDeque<String>,
    platform: String
) -> Result<(), String> {
    tokio::spawn(linked_helper( ubisoft_api, ctx, msg, args, platform, true ));

    Ok(())
}
async fn applications_helper(
    ubisoft_api: Arc<Mutex<Ubisoft>>,
    ctx: serenity::client::Context,
    msg: GenericMessage,
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
        msg.channel_id, 
        title, 
        &body, 
        &format!("https://ubisoft-avatars.akamaized.net/{account_id}/default_tall.png")
    ).await
        .unwrap();

    info!("Result: {res}");

    Ok(())
}
fn serialize_applications_response (
    res: &Value
) -> Result<String> {
    let application_ids = HashMap::from([
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
    msg: GenericMessage,
    args: VecDeque<String>
) -> Result<(), String> {
    tokio::spawn(linked( backend_handles.ubisoft_api, ctx, msg, args, String::from("uplay")));

    Ok(())
}
pub async fn lookup_xbox(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: GenericMessage,
    args: VecDeque<String>
) -> Result<(), String> {
    tokio::spawn(linked( backend_handles.ubisoft_api, ctx, msg, args, String::from("xbl")));

    Ok(())
}
pub async fn lookup_psn(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: GenericMessage,
    args: VecDeque<String>
) -> Result<(), String> {
    tokio::spawn(linked( backend_handles.ubisoft_api, ctx, msg, args, String::from("psn")));

    Ok(())
}
pub async fn applications(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: GenericMessage,
    args: VecDeque<String>
) -> Result<(), String> {
    applications_helper( backend_handles.ubisoft_api, ctx, msg, args ).await
}
pub fn recon_helper_severity_emoji(
    bad: u8
) -> String {
    match bad {
        0 => String::from("🟢"),
        1 => String::from("🟡"),
        2 => String::from("🔴"),
        _ => String::from("❔")
    }
}
pub fn recon_helper_queued_with<'a>(
    uplay_username: &'a str,
    account_id: &'a str
) -> Result<String, String> {
    // Query the `Stats.CC` website and get the raw HTML
    let html = ureq::get(&format!(
            "https://stats.cc/siege/{uplay_username}/{account_id}/playedWith"
        ))
        .call().map_err(|e| format!("{e:#?}"))?
        .into_string().map_err(|e| format!("{e:#?}"))?;
    
    // Parse the HTML
    let document: Html = Html::parse_document(&html);

    // Build a selector for the class `grid grid-cols-1 gap-1`
    let selector = Selector::parse(".grid.grid-cols-1.gap-1")
        .map_err(|e| anyhow!("{e:#?}")).map_err(|e| format!("{e:#?}"))?;

    // Compile the rough rank points
    let ranks: HashMap<&str, u32> = HashMap::from([
        ("champion-small", 4500),
        ("diamond-1-small", 4400),
        ("diamond-2-small", 4300),
        ("diamond-3-small", 4200),
        ("diamond-4-small", 4100),
        ("diamond-5-small", 4000),
        ("emerald-1-small", 3900),
        ("emerald-2-small", 3800),
        ("emerald-3-small", 3700),
        ("emerald-4-small", 3600),
        ("emerald-5-small", 3500),
        ("platinum-1-small", 3400),
        ("platinum-2-small", 3300),
        ("platinum-3-small", 3200),
        ("platinum-4-small", 3100),
        ("platinum-5-small", 3000),
        ("gold-1-small", 2900),
        ("gold-2-small", 2800),
        ("gold-3-small", 2700),
        ("gold-4-small", 2600),
        ("gold-5-small", 2500),
        ("silver-1-small", 2400),
        ("silver-2-small", 2300),
        ("silver-3-small", 2200),
        ("silver-4-small", 2100),
        ("silver-5-small", 2000),
        ("bronze-1-small", 1900),
        ("bronze-2-small", 1800),
        ("bronze-3-small", 1700),
        ("bronze-4-small", 1600),
        ("bronze-5-small", 1500),
        ("copper-1-small", 1400),
        ("copper-2-small", 1300),
        ("copper-3-small", 1200),
        ("copper-4-small", 1100),
        ("copper-5-small", 1000)
    ]);
    let rps: HashMap<u32, &str> = HashMap::from([
        (4500, "Champion"),
        (4400, "Diamond 1"),
        (4300, "Diamond 2"),
        (4200, "Diamond 3"),
        (4100, "Diamond 4"),
        (4000, "Diamond 5"),
        (3900, "Emerald 1"),
        (3800, "Emerald 2"),
        (3700, "Emerald 3"),
        (3600, "Emerald 4"),
        (3500, "Emerald 5"),
        (3400, "Platinum 1"),
        (3300, "Platinum 2"),
        (3200, "Platinum 3"),
        (3100, "Platinum 4"),
        (3000, "Platinum 5"),
        (2900, "Gold 1"),
        (2800, "Gold 2"),
        (2700, "Gold 3"),
        (2600, "Gold 4"),
        (2500, "Gold 5"),
        (2400, "Silver 1"),
        (2300, "Silver 2"),
        (2200, "Silver 3"),
        (2100, "Silver 4"),
        (2000, "Silver 5"),
        (1900, "Bronze 1"),
        (1800, "Bronze 2"),
        (1700, "Bronze 3"),
        (1600, "Bronze 4"),
        (1500, "Bronze 5"),
        (1400, "Copper 1"),
        (1300, "Copper 2"),
        (1200, "Copper 3"),
        (1100, "Copper 4"),
        (1000, "Copper 5")
    ]);

    // Build the body of the message
    let mut body = String::new();

    // Find all elements that match the selector
    let mut elements = document.select(&selector);
    if let Some(played_with_grid) = elements.next() {
        let mut played_with_players = Vec::new();

        // Convert each player item to a `PlayedWithPlayer`
        for player in played_with_grid.child_elements() {
            // Check if the player has a reputation
            //  ban or actual ban
            let has_rep_ban = player.attr("class")
                .map(|st| st.contains("ring-warning"))
                .unwrap_or(false);
            let has_real_ban = player.attr("class")
                .map(|st| st.contains("ring-error"))
                .unwrap_or(false);

            // Extract any useful information
            let mut useful_text = player.text()
                .map(|st| st.trim())
                .filter(|st| st.len() > 0 && *st != "/" )
                .collect::<Vec<&str>>();

            // Find and extract the image from the children 
            //  (there is only one image per player)
            let rank = player.select(&Selector::parse("image").unwrap())
                .next()
                .and_then(|img| img.attr("href"))
                .and_then(|img_src| {
                    Some(img_src
                        .replace("https://static.stats.cc/siege/ranks/", "")
                        .replace(".webp", ""))
                })
                .and_then(|rank_str| {
                    ranks.get(rank_str.as_str())
                        .and_then(|rank| Some(*rank))
                });

            // If they're champ, ignore the first argument
            if rank == Some(4500) {
                useful_text.remove(0);
            }

            // Convert the useful text to a `PlayedWithPlayer`
            if let Some(played_with_player) = PlayedWithPlayer::from(
                useful_text.clone(),
                has_rep_ban,
                has_real_ban,
                rank
            ) {
                played_with_players.push(played_with_player);
            } else {
                eprintln!("Failed to convert player: {:#?}", useful_text);
            }
        }

        // Statistically analyze the players to find the average lobby rank
        let mut total_games_played: u32 = 0;
        let average_lobby = played_with_players.iter()
            .map(|player| {
                total_games_played += player.games_played;

                player.rank.unwrap_or(0) *
                    player.games_played as u32 
            })
            .sum::<u32>() as f64 / total_games_played as f64;
        let average_lobby_rounded = average_lobby.round() as u32 / 100 * 100;
        let average_lobby_readable = rps.get(&average_lobby_rounded)
            .unwrap_or(&"Unknown");

        // Count the number of cheaters they queued with,
        //  what percentage of the lobby they make up,
        //  and how many games they played together
        let cheaters = played_with_players.iter()
            .filter(|player| player.hard_ban || player.rep_ban)
            .count();
        let cheater_games_played = played_with_players.iter()
            .filter(|player| player.hard_ban || player.rep_ban)
            .map(|player| player.games_played)
            .sum::<u32>();
        let cheater_percentage = cheater_games_played as f64 / total_games_played as f64 * 100.0;

        let is_rank_severe = recon_helper_severity_emoji((average_lobby_rounded < 3000) as u8 + (average_lobby_rounded < 2000) as u8);
        let is_cheater_count_severe = recon_helper_severity_emoji((cheaters > 2) as u8 + (cheaters > 5) as u8);
        let is_cheater_percentage_severe = recon_helper_severity_emoji((cheater_percentage > 2.5) as u8 + (cheater_percentage > 5.0) as u8);
        let is_cheater_games_played_severe = recon_helper_severity_emoji((cheater_games_played > 10) as u8 + (cheater_games_played > 20) as u8);
        
        body += &format!("### Average Lobby Rank\n- {is_rank_severe} **{average_lobby_rounded}** ({average_lobby_readable})\n");
    
        if cheaters > 0 {
            body += &format!("### Distinct Friendly Cheaters Queued With\n- {is_cheater_count_severe} **{cheaters}** cheaters\n- {is_cheater_percentage_severe} **{cheater_percentage:.2}%** of friendly teammates\n");
            body += &format!("### Games Played With Friendly Cheater\n- {is_cheater_games_played_severe} **{cheater_games_played}** games played\n");
        }

        info!("Average Lobby Rank: {average_lobby_rounded} ({average_lobby_readable})");
        info!("Distinct Friendly Cheaters Queued With: {cheaters} ({cheater_percentage:.2}% of friendly teammates)");
        info!("Games Played With Friendly Cheater: {cheater_games_played}");
    } else {
        let no_recently_played_with_severity = recon_helper_severity_emoji(1);
        body += &format!("### No Recently Played With\n- {no_recently_played_with_severity} **Has not** recently played ranked\n");
    
        warn!("Failed to get recently played with data!");
    }


    
    // Build a selector for the attribute `aria-labelledby` as "Ubisoft Bans"
    let hard_ban_selector = Selector::parse("[aria-labelledby='Ubisoft Bans']")
        .map_err(|e| format!("{e:#?}"))?;

    // Get the text of the hard bans, if they exist
    let hard_ban_text = document.select(&hard_ban_selector)
        .next()
        .map(|el| {
            el.text().map(|st| st.trim().to_string()).collect::<Vec<String>>().join(" - ")
        });

    // Build a selector for the attribute `aria-labelledby` as "Reputation Bans"
    let rep_ban_selector = Selector::parse("[aria-labelledby='Reputation Bans']")
        .map_err(|e| format!("{e:#?}"))?;

    // Get the text of the rep ban, if they exist
    let rep_ban_text = document.select(&rep_ban_selector)
        .next()
        .map(|el| {
            el.text().map(|st| st.trim().to_string()).collect::<Vec<String>>().join(" - ")
        });

    info!("Ubisoft Bans: {hard_ban_text:#?}");
    info!("Reputation Bans: {rep_ban_text:#?}");

    let is_rep_ban_severe = recon_helper_severity_emoji(rep_ban_text.is_some() as u8 * 2);
    let is_hard_ban_severe = recon_helper_severity_emoji(hard_ban_text.is_some() as u8 * 2);


    if let Some(hard_ban_text) = hard_ban_text {
        body += &format!("### Hard Bans\n- {is_hard_ban_severe} {hard_ban_text}\n");
    }

    if let Some(rep_ban_text) = rep_ban_text {
        body += &format!("### Reputation Bans\n- {is_rep_ban_severe} {rep_ban_text}\n");
    }

    Ok(body)
}
pub async fn recon(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: GenericMessage,
    args: VecDeque<String>
) -> Result<(), String> {
    let title = "OPSEC - Recon";
    let mut body = String::from("## 🕵️ Recon\n\n");

    // Ensure input argument
    let mut account_id = args
        .into_iter()
        .collect::<Vec<String>>()
        .join(" ");
    if account_id == "" {
        return Err(String::from("Please supply an account ID or username!"));
    }

    // Ensure that input is an account ID
    account_id = backend_handles.ubisoft_api
        .lock().await
        .get_account_id(account_id.clone(), String::from("uplay")).await
        .map_err(|_| format!("Account **{account_id}** does not exist!"))?;

    // Get profiles
    let profiles: Vec<Value> = get_profiles( backend_handles.ubisoft_api.clone(), &account_id )
        .await
        .ok_or(format!("Account ID **{account_id}** does not exist!"))?;
    let uplay_username = profiles.iter()
        .find(|profile| profile["platformType"].as_str() == Some("uplay"))
        .and_then(|profile| profile["nameOnPlatform"].as_str())
        .ok_or(String::from("Supplied account does not have a Uplay account!"))?;
    let steam_id = profiles.iter()
        .find(|profile| profile["platformType"].as_str() == Some("steam"))
        .and_then(|profile| profile["idOnPlatform"].as_str());
    info!("Uplay Username: {uplay_username}");

    body += &format!("- **Uplay Username**: {uplay_username}\n");
    body += &format!("- **Account ID**: {account_id}\n");
    if let Some(steam_id) = steam_id {
        body += &format!("- **Steam ID**: {steam_id}\n");
    }

    match recon_helper_queued_with(
        uplay_username,
        &account_id
    ) {
        Ok(recon_body) => body += &recon_body,
        Err(_) => {
            // Wait a second and try again
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            match recon_helper_queued_with(
                uplay_username,
                &account_id
            ) {
                Ok(recon_body) => body += &recon_body,
                Err(e) => {
                    let no_recently_played_with_severity = recon_helper_severity_emoji(3);
                    body += &format!("### Failed To Parse Stats.CC data\n- {no_recently_played_with_severity} Likely means Stats.CC is down.\n");
                
                    warn!("Failed to get recently played with data!\n\nError {e:#?}");
                }
            }
        }
    }

    if let Some(steam_id) = steam_id {
        let steamidfinder_response_raw = ureq::get(&format!(
                "https://api.findsteamid.com/steam/api/summary/{steam_id}"
            ))
            .set("Accept", "application/json")
            .call().map_err(|e| format!("{e:#?}"))?
            .into_string().map_err(|e| format!("{e:#?}"))?;

        let steamidfinder_response_value: Value = serde_json::from_str(&steamidfinder_response_raw)
            .map_err(|e| format!("{e:#?}"))?;

        if let Some(ban_value) = steamidfinder_response_value
            .get(0)
            .and_then(|val| val.get("ban")) {
            let ban_data: FindSteamIDBanInfo = serde_json::from_value(ban_value.clone())
                .map_err(|e| format!("{e:#?}"))?;

            if ban_data.communitybanned || ban_data.vacbanned || ban_data.numberofvacbans > 0 || ban_data.economyban != "none" {
                body += &format!("### Steam Bans\n");

                if ban_data.communitybanned {
                    let severity = recon_helper_severity_emoji(1);
                    body += &format!("- {severity} Community Banned\n");
                }
                if ban_data.economyban != "none" {
                    let severity = recon_helper_severity_emoji(1);
                    body += &format!("- {severity} Economy Banned\n");
                }
                if ban_data.vacbanned {
                    let severity = recon_helper_severity_emoji(2);
                    body += &format!("- {severity} **VAC Banned**\n");
                    body += &format!("- {severity} **{}** VAC Bans\n", ban_data.numberofvacbans);
                }
            } else {
                let severity = recon_helper_severity_emoji(0);
                body += &format!("### Steam Linked\n- {severity} **No** Bans\n");
            }
            

            info!("Steam Ban Data: {ban_data:#?}");
        }
    } else {
        let severity = recon_helper_severity_emoji(3);
        body += &format!("### Steam Linked\n- {severity} **No** Linked Steam Account\n");
    }

    // Send the final result
    send_embed_no_return(
        ctx.clone(), 
        msg.channel_id.clone(), 
        title, 
        &body.clone(), 
        &format!("https://ubisoft-avatars.akamaized.net/{account_id}/default_tall.png")
    ).await
        .unwrap();

    Ok(())
}
pub async fn mosscheck(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: GenericMessage,
    args: VecDeque<String>
) -> Result<(), String> {
    // Load `assets/logfile.log` into a string
    let log = args
        .into_iter()
        .collect::<Vec<String>>()
        .join(" ")
        .split("\n")
        .enumerate()
        .map(|(ind, st)| format!("{} - {}", ind + 1, st))
        .collect::<Vec<String>>()
        .join("\n");
    info!("Loaded log!");

    let prompt = "You are a black hat reverse engineer and cheat developer fluent in evading kernel anti-cheats - in this case, BattlEye.

        Siege cheaters are among the smartest, and often use ingenious methods like DMA cards paired with second PCs, internal cheats with pre-signed drivers and surprising methods of injection, or even external cheats with well-disguised profiles.

        You're unconcerned with harmless software. Find the good stuff, I'm talking:
        - Secure boot disabled
        - Windows Defender is disabled
        - Unusual syscalls
        - Graphics pipelines being hooked
        - Injection and codecaving

        The line between a cheat and malware is thin. Look for common cybersecurity exploits in system drivers. The only bannable cheat type is wallhacks, which obviously have to be rendered on the player's screen somehow. Those are the two things to look for - how is it reading and writing Siege's memory, and how is it being rendered.

        Remember, if there are any cheats present, they obviously isn't detected by BattlEye. If you provide concrete for a cheat and it's faceitous, you are banning an innocent player. Don't slip up, but don't miss genuine red flags. Oh, and remember - you verified all the SHA hashes already. Don't bother with those.

        Template response:
        {
                \"cheating\": true | false
                \"concrete_evidence\": [
                        {
                            \"line_numbers\": \"#-#\",
                            \"cheat_type\": \"EXTERNAL-CHEAT\" | \"INTERNAL-CHEAT\" | \"DMA-CARD\",
                            \"confidence\": \"DEFINITIVE-PROOF\" | \"SPECULATIVE\",
                            \"reasoning\": \"...\"
                        }
                ]
        }";



    let ubisoft_id_re = Regex::new(r"(\\([^\\]+?)\\GameSettings)")
        .expect("Hardcoded regex failed to compile");

    // Compile all Ubisoft IDs from the log file
    let mut ubisoft_ids = HashSet::new();
    for capture in ubisoft_id_re.captures_iter(&log) {
        if let Some(ubisoft_id) = capture.get(2) {
            ubisoft_ids.insert(ubisoft_id.as_str());
        }
    }
    if ubisoft_ids.len() > 0 {
        // Run both the `recon` and `linked` commands on each Ubisoft ID
        info!("Extracted the following Ubisoft IDs: {ubisoft_ids:#?}");
    
        // Start the recon commands
        send_embed_no_return(
            ctx.clone(), 
            msg.channel_id.clone(), 
            "Step 1/3 - Recon", 
            "Performing recon for various suspicious flags...", 
            get_random_anime_girl()
        ).await
            .unwrap();
    
        let mut join_handles = Vec::new();
        for ubisoft_id in ubisoft_ids.clone() {
            let mut args = VecDeque::new();
            args.push_back(ubisoft_id.to_string());
    
            join_handles.push(tokio::spawn(recon( backend_handles.clone(), ctx.clone(), msg.clone(), args.clone())));
        }
        // Wait for all the `recon` commands to finish
        for handle in join_handles {
            if let Err(e) = handle.await.map_err(|e| format!("{e:#?}"))? {
                warn!("Failed to run `recon` command: {e:#?}");
            };
        }
    
    
        // Sleep for 200ms to avoid rate limits
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        // Start the linked commands
        send_embed_no_return(
            ctx.clone(), 
            msg.channel_id.clone(), 
            "Step 2/3 - Linking", 
            "Collecting linked data on all played on accounts...", 
            get_random_anime_girl()
        ).await
            .unwrap();
        let mut join_handles = Vec::new();
        for ubisoft_id in ubisoft_ids.clone() {
            let mut args = VecDeque::new();
            args.push_back(ubisoft_id.to_string());
            
            join_handles.push(tokio::spawn(linked_helper( backend_handles.clone().ubisoft_api, ctx.clone(), msg.clone(), args.clone(), String::from("uplay"), false)));
        }
        // Wait for all the `linked` commands to finish
        for handle in join_handles {
            if let Err(e) = handle.await.map_err(|e| format!("{e:#?}"))? {
                warn!("Failed to run `linked` command: {e:#?}");
            };
        }
    } else {
        // Sleep for 200ms to avoid rate limits
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        if ubisoft_ids.len() == 0 {
            // Warn the user
            send_embed_no_return(
                ctx.clone(), 
                msg.channel_id.clone(), 
                "No Ubisoft IDs Found!", 
                "Skipping steps 1 and 2.\n\nThis likely means the user did not launch Siege, you submitted an invalid MOSS file, or that they have their GameSettings files stored in OneDrive.\n\nIn this case, it is advisable to directly PC check the user - it is not very common for people to do this.", 
                get_random_anime_girl()
            ).await
                .unwrap();
        }
    }





    // Query GPT-4o for additional insights on the file itself
    send_embed_no_return(
        ctx.clone(), 
        msg.channel_id.clone(), 
        "Step 3/3 - Recon", 
        "Analyzing file for suspicious flags...", 
        get_random_anime_girl()
    ).await
        .unwrap();

    let open_ai_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let response = ureq::post("https://api.openai.com/v1/chat/completions")
        .set("Authorization", &("Bearer ".to_owned() + &open_ai_key))
        .set("Content-Type", "application/json")
        .send_json(serde_json::json!({
            "model": "gpt-4o-mini",
            "messages": [
                {
                    "role": "system",
                    "content": prompt
                },
                {
                    "role": "user",
                    "content": &log
                }
            ],
            "temperature": 0,
            "max_tokens": 2670,
            "top_p": 1,
            "frequency_penalty": 0,
            "presence_penalty": 0
        }));
    match response {
        Ok(response) => {
            let response: Value = response.into_json()
                .map_err(|e| format!("{e:#?}"))?;

            // Extract the response value
            let response = response["choices"]
                .get(0)
                .and_then(|val| val["message"]["content"].as_str())
                .and_then(|st| Some(st.replace("```json","").replace("```","")))
                .ok_or(String::from("Failed to get response!"))?;
            info!("Response: {response:#?}");

            let response: Value = serde_json::from_str::<Value>(&response)
                .map_err(|e| format!("{e:#?}"))?;
            info!("Response: {response:#?}");

            // Deserialize it
            match serde_json::from_value::<GPTResponse>(response) {
                Ok(response) => {
                    if !response.cheating {
                        send_embed_no_return(
                            ctx.clone(), 
                            msg.channel_id.clone(), 
                            "GPT-4o Analysis", 
                            "No cheating was detected in the file.",
                            get_random_anime_girl()
                        ).await
                            .unwrap();
                    }
        
                    let mut body = response.concrete_evidence.into_iter()
                        .filter_map(|flag| {
                            if flag.confidence != "DEFINITIVE_PROOF" && flag.confidence != "DEFINITIVE-PROOF" {
                                return None;
                            }
        
                            Some(format!(
                                "### Potential {}\n- **Lines**: {}\n- **Reasoning**: {}\n",
                                flag.cheat_type,
                                flag.line_numbers,
                                flag.reasoning
                            ))
                        })
                        .collect::<String>();
                    if body.len() > 0 {
                        body += "\n\n*Always take model output with a grain of salt, as it assume the player IS cheating. If the model sees anything extremely suspicious, these are not grounds to ban on. Instead, contact someone knowledgeable to then manually check.*";
                    } else {
                        body += "No definitive proof of cheating was found.\n\nIf you still believe the player is cheating, contact someone knowledgable to manually check them.";
                    }

                    send_embed_no_return(
                        ctx.clone(), 
                        msg.channel_id.clone(), 
                        "Model Analysis", 
                        &body,
                        get_random_anime_girl()
                    ).await
                        .unwrap();

                },
                Err(err) => {
                    send_embed_no_return(
                        ctx.clone(), 
                        msg.channel_id.clone(), 
                        "Model Analysis Failed", 
                        &format!("{err:#?}"),
                        get_random_anime_girl()
                    ).await
                        .unwrap();

                }
            }


        },
        Err(e) => {
            warn!("Failed to query GPT-4o: {e:#?}");

            send_embed_no_return(
                ctx.clone(), 
                msg.channel_id.clone(), 
                "GPT-4o Analysis", 
                "Failed to query model for additional insights on the file.",
                get_random_anime_girl()
            ).await
                .unwrap();
        }
    }



    // Sleep for 200ms to avoid rate limits
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    // Send the final result
    send_embed_no_return(
        ctx.clone(), 
        msg.channel_id.clone(), 
        "Done!", 
        "Automatic MOSS check complete.", 
        get_random_anime_girl()
    ).await
        .unwrap();

    Ok(())
}

pub async fn build_opsec_commands() -> R6RSCommand {
    let mut opsec_nest_command = R6RSCommand::new_root(
        String::from("Commands for location information on Ubisoft Connect accounts."),
        String::from("OPSEC")
    );
    opsec_nest_command.attach(
        String::from("pc"),
        R6RSCommand::new_leaf(
            String::from("Looks up a Ubisoft account based on their registered PC username."),
            AsyncFnPtr::new(lookup_pc),
            vec!(vec!(String::from("username"))),
            Some(String::from("opsec"))
        )
    );
    opsec_nest_command.attach(
        String::from("xbox"),
        R6RSCommand::new_leaf(
            String::from("Looks up a Ubisoft account based on their registered Xbox username."),
            AsyncFnPtr::new(lookup_xbox),
            vec!(vec!(String::from("username"))),
            Some(String::from("opsec"))
        )
    );
    opsec_nest_command.attach(
        String::from("psn"),
        R6RSCommand::new_leaf(
            String::from("Looks up a Ubisoft account based on their registered PSN username."),
            AsyncFnPtr::new(lookup_psn),
            vec!(vec!(String::from("username"))),
            Some(String::from("opsec"))
        )
    );
    opsec_nest_command.attach(
        String::from("applications"),
        R6RSCommand::new_leaf(
            String::from("Looks up a Ubisoft account based on their username (PC only)."),
            AsyncFnPtr::new(applications),
            vec!(vec!(String::from("username"))),
            Some(String::from("opsec"))
        )
    );
    opsec_nest_command.attach(
        String::from("recon"),
        R6RSCommand::new_leaf(
            String::from("Analyzes a Ubisoft account for suspicious behaviour based on their username (PC only)."),
            AsyncFnPtr::new(recon),
            vec!(vec!(String::from("username"))),
            Some(String::from("opsec"))
        )
    );
    opsec_nest_command.attach(
        String::from("mosscheck"),
        R6RSCommand::new_leaf(
            String::from("Runs a complete suspicion check on a provided MOSS file."),
            AsyncFnPtr::new(mosscheck),
            vec!(vec!(String::from("file"))),
            Some(String::from("opsec"))
        )
    );

    startup!("OPSEC commands have been built.");

    opsec_nest_command
}