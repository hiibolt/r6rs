use crate::Message;
use crate::Context;
use crate::send_embed;

pub async fn help( 
    ctx: Context,
    msg: Message 
) {
    let _ = send_embed(
        &ctx, 
        &msg, 
        "R6 - All Sections - Help", 
        &(String::from("**Economy Command List**:\n- `r6 econ analyze <item name | item id>`\n- `r6 econ graph <item name | item id>`\n- `r6 econ profit <purchased at> <item name | item id>`\n- `r6 econ list <(optional) page #>`\n- `r6 econ help`\n\n") +
            "**OPSEC Command List**:\n- `r6 opsec <pc | xbox | psn> <account name>`\n- `r6 opsec namefind <username1> <username2> ...`\n- `r6 opsec help`\n\n" +
            "**OSINT Command List**:\n- `osint email <email>`\n- `osint username <username>`\n- `osint ip <ip>`\n- `osint password <password>`\n- `osint name <name>`\n- `osint last_ip <last_ip>`\n\n" +
            "**Ban Watch Command List**:\n- **Still under development, stay cozy...**\n\n" +
            "**Admin Command List**:\n- `r6 admin whitelist <section> <user id>`\n- `r6 admin blacklist <section> <user id>`\n- `r6 admin help`\n\n\n*Developed by @hiibolt on GitHub*"),
        "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4"
    ).await
        .expect("Failed to send embed!");
}