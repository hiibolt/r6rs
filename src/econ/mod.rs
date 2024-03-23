use crate::VecDeque;
use crate::Message;
use crate::Context;
use crate::State;
use crate::Mutex;
use crate::Arc;
use crate::unimplemented;
use crate::send_embed;

async fn name_or_item_id( state: Arc<Mutex<State>>, unknown_id: String ) -> String {
    if unknown_id.chars().filter(|&ch| ch.is_ascii_digit() ).count() > 5 {
        println!("probably an id");
        return unknown_id;
    }
    println!("{:?}", state.lock().await.market_data);
    unknown_id
}
async fn list( state: Arc<Mutex<State>>, mut args: VecDeque<String> ) -> String {
    // Get the page number
    let page: usize = args.pop_front()
        .and_then(|st| st.parse::<usize>().ok() )
        .unwrap_or(1);

    let mut msg: String = format!("# Ask Bolt for new items.\n\n## Skins (Page {page}):\n(Run `r6 econ list {}` to see the next page)\n\n", page + 1);
    
    let mut count: u8 = 0;
    for (key, _) in state.lock().await.id_list
        .iter()
        .skip( (page - 1) * 25 ) // Handle 'pages'
        .take( 25 )
    {
        msg += &format!("{key}\n");

        count += 1;
    }

    msg
}
async fn data( state: Arc<Mutex<State>>, mut args: VecDeque<String> ) -> Result<String, String> {
    let mut msg: String = format!("");

    let item_id = name_or_item_id(
        state.clone(),
        args.pop_front()
            .ok_or(String::from("Missing last argument `item id`!"))?
    ).await;

    println!("");

    Ok(msg)
}
pub async fn econ( state: Arc<Mutex<State>>, ctx: Context, msg: Message, mut args: VecDeque<String> ) {
    match args
        .pop_front()
        .unwrap_or(String::from("help"))
        .as_str()
    {
        "list" => {
            let result: String = list( state, args ).await;

            send_embed(
                &ctx, 
                &msg, 
                "Tracked Skins", 
                &result, 
                "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
            ).await
                .unwrap();
        },
        "analyze" => {
            let result: String = data( state, args )
                .await
                .unwrap_or_else(|err| err);

            send_embed(
                &ctx, 
                &msg, 
                "Tracked Skins", 
                &result, 
                "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
            ).await
                .unwrap();
        },
        "help" => {
            unimplemented( ctx, msg, "help" ).await;
        },
        nonexistant => {
            send_embed(
                &ctx, 
                &msg, 
                "Command does not exist", 
                &format!("The command **{nonexistant}** is not valid!"), 
                "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
            ).await
                .unwrap();
        }
    }
}