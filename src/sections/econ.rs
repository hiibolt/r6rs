use crate::{
    apis::Ubisoft,
    helper::{
        lib::{get_random_anime_girl, send_embed_no_return, AsyncFnPtr}, 
        bot::{BackendHandles, State, GenericMessage}, 
        command::R6RSCommand
    },
    info, startup,
    VecDeque,
    Colorize,
    Mutex, Arc
};

use std::time::{
    SystemTime,
    UNIX_EPOCH
};

use serenity::all::{
    CreateMessage,
    CreateEmbed,
    CreateAttachment
};
use plotpy::{
    Curve,
    Plot,
};

async fn name_or_item_id( state: Arc<Mutex<State>>, unknown_id: String ) -> Result<String, String> {
    if unknown_id.len() == 0 {
        return Err(String::from("Missing the `item_id` argument!\n\nRun `>>r6 econ help` if you're confused."));
    }
    
    if unknown_id.chars().filter(|&ch| ch.is_ascii_digit() ).count() > 5 {
        return Ok(unknown_id);
    }

    Ok(state
        .lock().await
        .id_list
        .get(&unknown_id)
        .ok_or(format!("We aren't tracking the name `{unknown_id}`! If you think we are, try the ID instead."))?
        .as_str().to_owned())
}
async fn data( state: Arc<Mutex<State>>, args: VecDeque<String> ) -> Result<(String, String, String), String> {
    let mut msg: String = format!("");

    let item_id = name_or_item_id(
            state.clone(),
            args.into_iter()
                .collect::<Vec<String>>()
                .join(" ")
        ).await?;
    
    // Grab the item data
    let item_data = state
        .lock().await
        .market_data
        .get(&item_id)
        .ok_or(format!("We aren't tracking the item ID/item name `{item_id}`. Please request that @hiibolt add it!"))?
        .clone();

    // Grab a copy  of the sold data
    let item_sold_data: Vec<serde_json::Value> = item_data
        .get("sold").ok_or(String::from("Couldn't retrieve data! Contact @hiibolt if you can see this."))?
        .as_array().ok_or(String::from("Couldn't retrieve data! Contact @hiibolt if you can see this."))?
        .clone();
    
    // Remove null sales
    let filtered_data: Vec<Vec<f64>> = item_sold_data
        .iter()
        .flat_map(|data_point| {
            if let Some(data_point_as_arr) = data_point.as_array().clone() {
                if data_point_as_arr[0].is_null() || data_point_as_arr[1].is_null() {
                    return None;
                }

                return Some(vec!(data_point_as_arr[0].as_f64().unwrap(), data_point_as_arr[1].as_f64().unwrap()));
            }
            None
        })
        .collect();

    // Grab its general metadata
    let item_general_data = item_data
        .get("data")
        .and_then(|val| val.as_array())
        .ok_or(format!("We are tracking the item ID `{item_id}`, but we don't yet have data! If @hiibolt just added it, please allow 5 minutes for data to propogate."))?;
    let grab_str_or_placeholder = |index: usize| -> String {
        item_general_data
            .get(index)
            .and_then(|val| val.as_f64())
            .and_then(|num| Some(num.to_string()))
            .unwrap_or(String::from("???"))
    };

    // RAP and Tags Section
    let ten_rap: f64 = filtered_data
        .iter().take(10).fold(0f64, |acc, vc| acc + vc[0])
        /
        (filtered_data.iter().take(10).count() as f64).max(1f64);
    let hundred_rap: f64 = filtered_data
        .iter().take(100).fold(0f64, |acc, vc| acc + vc[0])
        /
        (filtered_data.iter().take(100).count() as f64).max(1f64);
    let all_time_rap: f64 = filtered_data
        .iter().fold(0f64, |acc, vc| acc + vc[0])
        /
        (filtered_data.iter().count() as f64).max(1f64);
    let tags = item_data
        .get("tags")
        .and_then(|val| {
            val.as_array()
        })
        .and_then(|arr| {
            Some(arr.iter().map(|val| format!("{:?}", val)).collect())
        })
        .unwrap_or(vec!(String::from("No tags found!")));
    let data_len = filtered_data.len();

    // Buyers Section
    let minimum_buyer = grab_str_or_placeholder(0);
    let maximum_buyer = grab_str_or_placeholder(1);
    let volume_buyers = grab_str_or_placeholder(2);

    // Sellers Section
    let minimum_seller = grab_str_or_placeholder(3);
    let maximum_seller = grab_str_or_placeholder(4);
    let volume_sellers = grab_str_or_placeholder(5);
    let last_sold = filtered_data
        .iter()
        .next()
        .and_then(|vc| Some(vc[0].to_string()))
        .unwrap_or(String::from("???"));

    // Quick Analysis Section
    let max_buyer_vs_min_seller = 
        minimum_seller.parse::<f64>().unwrap_or(0f64) - 
        maximum_buyer.parse::<f64>().unwrap_or(0f64);
    let last_sale_vs_min_buyer = 
        last_sold.parse::<f64>().unwrap_or(0f64) - 
        minimum_buyer.parse::<f64>().unwrap_or(0f64);

    

    msg += &format!("# Buy:\n\tMinimum Buyer: **{minimum_buyer}** R6 credits\n\tMaximum Buyer: **{maximum_buyer}** R6 credits\n\tVolume Buyers: **{volume_buyers}**\n");
    msg += &format!("# Sell:\n\tMinimum Seller: **{minimum_seller}** R6 credits\n\tMaximum Seller: **{maximum_seller}** R6 credits\n\tVolume Sellers: **{volume_sellers}**\n\tLast Sold: **{last_sold}**\n\n");
    msg += &format!("### Quick Analysis:\n\tHighest Buyer vs. Lowest Seller: **{max_buyer_vs_min_seller}** R6 credits\n\tLast Sale vs. Lowest Seller: **{last_sale_vs_min_buyer}** R6 credits\n");
    msg += &format!("### RAP:\n\t10 - **{ten_rap}**\n\t100 - **{hundred_rap}**\n\tAll Time - **{all_time_rap}**\n\n\t*(Total Data: {data_len})*\n### Tags:\n\n{:?}\n### Item ID:\n\t{item_id}", tags);
    
    let item_name = item_data
        .get("name")
        .and_then(|val| val.as_str())
        .unwrap_or("???");
    let item_type = item_data
        .get("type")
        .and_then(|val| val.as_str())
        .unwrap_or("???");
    let item_asset_url = item_data
        .get("asset_url")
        .and_then(|val| val.as_str())
        .unwrap_or("???");

    Ok((msg, format!("{item_name} ({item_type})"), item_asset_url.to_owned()))
}
async fn list(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: GenericMessage,
    mut args: VecDeque<String>
) -> Result<(), String> {
    // Get the page number
    let page: usize = args.pop_front()
        .and_then(|st| st.parse::<usize>().ok() )
        .unwrap_or(1);

    let mut body: String = format!("# Ask Bolt for new items.\n\n## Skins (Page {page}):\n(Run `r6 econ list {}` to see the next page)\n\n", page + 1);
    
    for (key, _) in backend_handles.state.lock().await.id_list
        .iter()
        .skip( (page - 1) * 25 ) // Handle 'pages'
        .take( 25 )
    {
        body += &format!("{key}\n");
    }

    send_embed_no_return(
        ctx, 
        msg.channel_id, 
        "Tracked Skins", 
        &body, 
        get_random_anime_girl()
    ).await
        .unwrap();

    Ok(())
}
async fn graph_helper(
    state: Arc<Mutex<State>>,
    args: VecDeque<String>
) -> Result<String, String> {
    let item_id = name_or_item_id(
        state.clone(),
        args.into_iter()
            .collect::<Vec<String>>()
            .join(" ")
    ).await?;

    // Grab the item data
    let item_data = state
        .lock().await
        .market_data
        .get(&item_id)
        .ok_or(format!("We aren't tracking the item ID/item name `{item_id}`. Please request that @hiibolt add it!"))?
        .clone();

    // Grab a copy  of the sold data
    let item_sold_data: Vec<serde_json::Value> = item_data
        .get("sold").ok_or(String::from("Couldn't retrieve data! Contact @hiibolt if you can see this."))?
        .as_array().ok_or(String::from("Couldn't retrieve data! Contact @hiibolt if you can see this."))?
        .clone();
    
    // Remove null sales
    let filtered_data: Vec<Vec<f64>> = item_sold_data
        .iter()
        .flat_map(|data_point| {
            if let Some(data_point_as_arr) = data_point.as_array().clone() {
                //todo!(); make this better
                if data_point_as_arr[0].is_null() || data_point_as_arr[1].is_null() {
                    return None;
                }

                return Some(vec!(data_point_as_arr[0].as_f64().unwrap(), data_point_as_arr[1].as_f64().unwrap()));
            }
            None
        })
        .collect();
    
    // Extract the time and price data
    let mut times: Vec<f64> = filtered_data
        .iter()
        .map(|arr| {
            let time_since = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs_f64() - arr[1];
            time_since / 3600f64 / 24f64
        })
        .collect();
    let mut prices: Vec<f64> = filtered_data
        .iter()
        .map(|arr| arr[0])
        .collect();
    times.reverse();
    prices.reverse();
    
    // Extract the item metadata
    let item_name = item_data
        .get("name")
        .and_then(|val| val.as_str())
        .unwrap_or("???");
    let item_type = item_data
        .get("type")
        .and_then(|val| val.as_str())
        .unwrap_or("???");
    
    // Define our data curve
    let mut data_curve = Curve::new();
    data_curve.draw(&times, &prices);
    
    // Define the output file path
    let item_path = format!("assets/{item_id}.png");
    let valid_name = item_name.replace("'", "\\'");
    
    // Plot our data curve
    Plot::new()
        .add(&data_curve)
        .set_title(&format!("{valid_name} ({item_type})"))
        .set_labels("Time (days ago)","Price (R6 Credits)")
        .save(&item_path)?;
    
    // Remove the associated python file
    tokio::fs::remove_file(format!("assets/{item_id}.py"))
        .await
        .map_err(|err| format!("{err:?}"))?;
    
    Ok(item_id)
}
async fn profit_helper( 
    state: Arc<Mutex<State>>,
    mut args: VecDeque<String> 
) -> Result<(String, String), String> {
    let mut msg = String::from("");

    // Grab positional arguments
    let purchase_price = args
        .pop_front()
        .ok_or(String::from("Missing `purchase price` positional argument!"))?
        .parse::<f64>()
        .map_err(|_| String::from("Could not parse the `purchase price` positional argument into a number!\n\nDid you accidentally flip the `purchase price` and `item name/id`?"))?;
    let item_id = name_or_item_id(
        state.clone(),
        args.into_iter()
            .collect::<Vec<String>>()
            .join(" ")
    ).await?;

    // Grab the item data
    let item_data = state
        .lock().await
        .market_data
        .get(&item_id)
        .ok_or(format!("We aren't tracking the item ID/item name `{item_id}`. Please request that @hiibolt add it!"))?
        .clone();

    // Grab a copy  of the sold data
    let item_sold_data: Vec<serde_json::Value> = item_data
        .get("sold").ok_or(String::from("Couldn't retrieve data! Contact @hiibolt if you can see this."))?
        .as_array().ok_or(String::from("Couldn't retrieve data! Contact @hiibolt if you can see this."))?
        .clone();
        
    // Remove null sales
    let filtered_data: Vec<Vec<f64>> = item_sold_data
        .iter()
        .flat_map(|data_point| {
            if let Some(data_point_as_arr) = data_point.as_array().clone() {
                if data_point_as_arr[0].is_null() || data_point_as_arr[1].is_null() {
                    return None;
                }

                return Some(vec!(data_point_as_arr[0].as_f64().unwrap(), data_point_as_arr[1].as_f64().unwrap()));
            }
            None
        })
        .collect();

    // Calculate various sale numbers
    let profitable_sell: f64 = 1.1f64 * purchase_price;
    let ten_rap: f64 = filtered_data
        .iter().take(10).fold(0f64, |acc, vc| acc + vc[0])
        /
        (filtered_data.iter().take(10).count() as f64).max(1f64);
    let current_net_gain = (ten_rap - purchase_price ) * 0.9f64;

    // Extract asset url
    let item_asset_url = item_data
        .get("asset_url")
        .and_then(|val| val.as_str())
        .unwrap_or("???");
    
    msg += &format!("\n### Purchased At:\n\t**{purchase_price}** R6 credits\n### Sale Price to Break Even:\n\t**{profitable_sell}** R6 credits\n### Current Net Gain if Sold:\n\t**{current_net_gain}** R6 credits");

    Ok((msg, item_asset_url.to_owned()))
}
pub async fn transfer (
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: GenericMessage,
    mut args: VecDeque<String> 
) -> Result<(), String> {
    /* let number_of_items = args.pop_front()
        .unwrap_or(String::from("15"))
        .parse::<usize>()
        .unwrap_or(15)
        .min(15); */
    let number_of_items = 15;

    let mut block_ubisoft_api = backend_handles.ubisoft_api.clone();
    let mut used_login = false;
    if let Some(email) = args.pop_front() {
        if let Some(password) = args.pop_front() {
            info!("Logging in with email: {email} and password: {password}");

            let temporary_ubisoft_api = Arc::new(Mutex::new(Ubisoft::new(email, password)));

            if let Err(err) = temporary_ubisoft_api.lock().await.login().await {
                send_embed_no_return(
                    ctx, 
                    msg.channel_id, 
                    &format!("R6 - Economy - {} Least Sold Items", number_of_items), 
                    &format!("Failed to get items with an error! Please see below:\n\n{:#?}", err), 
                    get_random_anime_girl()
                ).await
                    .expect("Failed to send embed!");
        
                return Ok(());
            }

            block_ubisoft_api = temporary_ubisoft_api;

            used_login = true;
        } else {
            send_embed_no_return(
                ctx, 
                msg.channel_id, 
                &format!("R6 - Economy - {} Least Sold Items", number_of_items), 
                "You provided an email, but no password! Please provide both to use the login feature.", 
                &get_random_anime_girl()
            ).await
                .expect("Failed to send embed!");

            return Ok(());
        }
    }

    let items = if used_login {
        block_ubisoft_api
            .lock().await
            .get_least_sold_owned(number_of_items).await
    } else {
        block_ubisoft_api
            .lock().await
            .get_least_sold(number_of_items).await
    };

    if let Err(err) = items {
        send_embed_no_return(
            ctx, 
            msg.channel_id, 
            &format!("R6 - Economy - {} Least Sold Items", number_of_items), 
            &format!("Failed to get items with an error! Please see below:\n\n{:#?}", err), 
            get_random_anime_girl()
        ).await
            .expect("Failed to send embed!");

        return Ok(());
    }

    let items = items.expect("Unreachable!");

    let mut body = String::new();

    for item in &items {
        body.push_str(
            &format!(
                "\n**{}** - [Sell](https://www.ubisoft.com/en-gb/game/rainbow-six/siege/marketplace?route=sell%2Fitem-details&itemId={})\n{}\n**{}** Current Sellers, Last Sold at **{}** R6 Credits\n",
                item.name,
                item.item_id,
                item.item_type,
                item.sellers,
                item.last_sold_at
            )
        );
    }

    if used_login {
        body.push_str("\n\nData was gathered using your provided Ubisoft account.");
    } else {
        body.push_str("\n\nData is global, and gathered using an arbitrary Ubisoft account.");
    }

    send_embed_no_return(
        ctx, 
        msg.channel_id, 
        &format!("R6 - Economy - {} Least Sold Items", number_of_items), 
        &body, 
        &items.get(0).expect("Unreachable?").asset_url
    ).await
        .expect("Failed to send embed!");

    Ok(())
}
pub async fn analyze(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: GenericMessage,
    args: VecDeque<String>
) -> Result<(), String> {
    let (body, title, item_img) = data( backend_handles.state, args )
        .await
        .unwrap_or_else(|err| 
            (err, String::from("Error!"), String::from(get_random_anime_girl()))
        );
    
    send_embed_no_return(
        ctx, 
        msg.channel_id, 
        &title, 
        &body, 
        &item_img,
    ).await
        .map_err(|e| format!("{e:#?}"))
}
pub async fn graph(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: GenericMessage,
    args: VecDeque<String>
) -> Result<(), String> {
    let item_id = graph_helper( backend_handles.state, args )
        .await?;

    let attachment = CreateAttachment::path(&format!("assets/{item_id}.png"))
            .await
            .expect("Failed to create attachment!");

    let embed = CreateEmbed::new()
        .image(format!("attachment://{item_id}.png"));

    let builder = CreateMessage::new()
        .embed(embed)
        .add_file(attachment);

    tokio::spawn(msg.channel_id.send_message(ctx.http, builder));

    Ok(())
}
pub async fn profit(
    backend_handles: BackendHandles,
    ctx: serenity::client::Context,
    msg: GenericMessage,
    args: VecDeque<String>
) -> Result<(), String> {
    let (body, asset_url) = profit_helper( backend_handles.state, args ).await?;

    send_embed_no_return(
        ctx, 
        msg.channel_id, 
        "Profit Analytics", 
        &body, 
        &asset_url
    ).await
        .expect("Failed to send embed!");
    
    Ok(())
}

pub async fn build_econ_commands() -> R6RSCommand {
    let mut econ_nest_command = R6RSCommand::new_root(
        String::from("Commands related to the Rainbow Six Siege Marketplace and its economy."),
        String::from("Economy")
    );
    econ_nest_command.attach(
        String::from("list"),
        R6RSCommand::new_leaf(
            String::from("Lists all available skins."),
            AsyncFnPtr::new(list),
            vec!(vec!(), vec!(String::from("page #"))),
            Some(String::from("econ"))
        )
    );
    econ_nest_command.attach(
        String::from("analyze"),
        R6RSCommand::new_leaf(
            String::from("Creates a detailed data sheet on an item."),
            AsyncFnPtr::new(analyze),
            vec!(vec!(String::from("item name | item id"))),
            Some(String::from("econ"))
        )
    );
    econ_nest_command.attach(
        String::from("graph"),
        R6RSCommand::new_leaf(
            String::from("Graphs the all-time history of an item."),
            AsyncFnPtr::new(graph),
            vec!(vec!(String::from("item name | item id"))),
            Some(String::from("econ"))
        )
    );
    econ_nest_command.attach(
        String::from("profit"),
        R6RSCommand::new_leaf(
            String::from("Calculates the amount you would make if you sold your item right now."),
            AsyncFnPtr::new(profit),
            vec!(vec!(String::from("$ bought for"), String::from("item name | item id"))),
            Some(String::from("econ"))
        )
    );
    econ_nest_command.attach(
        String::from("transfer"),
        R6RSCommand::new_leaf(
            String::from("Finds the items with the least sellers either globally or on the account with the provided login."),
            AsyncFnPtr::new(transfer),
            vec!(vec!(), vec!(String::from("ubisoft email"), String::from("ubisoft password"))),
            Some(String::from("econ"))
        )
    );

    startup!("Economy commands have been built.");

    econ_nest_command
}