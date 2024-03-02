use crate::VecDeque;
use crate::Message;
use crate::Context;
use crate::unimplemented;
use crate::send_embed;
use crate::UbisoftAPI;
use crate::State;
use crate::{ Arc, Mutex };

async fn linked( 
    ubisoft_api: Arc<Mutex<UbisoftAPI>>,
    ctx: Context,
    msg: Message,
    mut args: VecDeque<String>
) { 
    let mut body = String::new();
    let title = "OPSEC - Linked";

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
    
    println!("{account_id}");

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