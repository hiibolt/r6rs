use crate::apis::BulkVS;
use crate::apis::Database;
use crate::apis::Snusbase;
use crate::apis::Ubisoft;
use crate::Message;
use crate::State;
use crate::{ Arc, Mutex };
use crate::read_to_string;
use std::fs::OpenOptions;
use std::io::Write;
use tokio::time::{ sleep, Duration };
use serenity::model::colour::Colour;
use serenity::all::EditMessage;
use serenity::all::{ CreateEmbed, CreateMessage };
use rand::prelude::SliceRandom;
use anyhow::Result;
use futures::future::{Future, BoxFuture};
use std::collections::HashMap;
use anyhow::{ anyhow, bail, Context };
use async_recursion::async_recursion;
use std::collections::VecDeque;

#[derive(Clone)]
pub struct BackendHandles {
    pub ubisoft_api: Arc<Mutex<Ubisoft>>,
    pub snusbase:    Arc<Mutex<Snusbase>>,
    pub bulkvs:      Arc<Mutex<BulkVS>>,
    pub state:       Arc<Mutex<State>>,
    pub database:    Arc<Mutex<Database>>
}
pub struct AsyncFnPtr<R> {
    func: Box<dyn Fn(
        BackendHandles,
        serenity::client::Context,
        Message,
        VecDeque<String>
    ) -> BoxFuture<'static, R> + Send + Sync + 'static>
}
impl <R> AsyncFnPtr<R> {
    pub fn new<F>(
        f: fn(
            BackendHandles,
            serenity::client::Context,
            Message,
            VecDeque<String>
        ) -> F
    ) -> AsyncFnPtr<F::Output> 
    where 
        F: Future<Output = R> + Send + Sync + 'static
    {
        AsyncFnPtr {
            func: Box::new(move |backend_handles, ctx, msg, args| Box::pin(f(backend_handles, ctx, msg, args))),
        }
    }
    pub async fn run(
        &self,
        backend_handles: BackendHandles,
        ctx: serenity::client::Context,
        msg: Message,
        args: VecDeque<String>
    ) -> R { 
        (self.func)(backend_handles, ctx, msg, args).await
    }
}
pub struct R6RSLeafCommand {
    pub function: AsyncFnPtr<Result<(), String>>,
    pub required_authorization: Option<String>,
    pub valid_args: Vec<Vec<String>>
}
pub struct R6RSRootCommand {
    pub commands: HashMap<String, Box<R6RSCommand>>,
    pub section_title: String
}
pub enum R6RSCommandType {
    RootCommand(R6RSRootCommand),
    LeafCommand(R6RSLeafCommand)
}
pub struct R6RSCommand
{
    pub inner: R6RSCommandType,
    pub description: String,
}
impl R6RSCommand {
    pub fn new_root(
        description: String,
        section_title: String
    ) -> R6RSCommand {
        R6RSCommand {
            inner: R6RSCommandType::RootCommand(R6RSRootCommand{ commands: HashMap::new(), section_title }),
            description
        }
    }
    pub fn new_leaf(
        description: String,
        function: AsyncFnPtr<Result<(), String>>,
        valid_args: Vec<Vec<String>>,
        required_authorization: Option<String>
    ) -> R6RSCommand {
        R6RSCommand {
            inner: R6RSCommandType::LeafCommand(R6RSLeafCommand { function, required_authorization, valid_args }),
            description
        }
    }

    pub fn attach(
        &mut self,
        name: String,
        command: R6RSCommand
    ) {
        match &mut self.inner {
            R6RSCommandType::RootCommand(root_command) => {
                root_command.commands.insert(name, Box::new(command));
            },
            _ => panic!("Cannot attach a command to a leaf command!")
        }
    }

    #[async_recursion]
    pub async fn print_help(
        &mut self,
        prefix: String,
        level: usize,
        github_friendly: bool
    ) -> String {
        let mut body = String::from("\n");

        let R6RSRootCommand{ commands, section_title } = if let R6RSCommandType::RootCommand(root_command) = &mut self.inner {
            root_command
        } else {
            panic!("Cannot print help for a leaf command!");
        };

        let mut subsection_count: usize = 0;
        // Handle subsections first
        body += &format!("{} {section_title}\n", "#".repeat(level));
        for (name, command) in commands.iter_mut() {
            match &command.inner {
                R6RSCommandType::RootCommand(_) => {
                    subsection_count += 1;
                    let nested_print = command.print_help(
                        prefix.clone() + " " + &name,
                        level + 1,
                        github_friendly
                    ).await;
                    
                    body += &nested_print;
                },
                _ => ()
            }
        }
        // Handle leaf commands
        let mut leaf_body = String::new();
        for (name, command) in commands.iter_mut() {
            match &command.inner {
                R6RSCommandType::LeafCommand(R6RSLeafCommand{required_authorization: _, valid_args, function: _}) => {
                    let description = command.description.to_owned();

                    for arg_set in valid_args {
                        leaf_body.push_str(&format!("\n`{prefix} {name}"));

                        for arg in arg_set {
                            leaf_body += &format!(" <{}>", arg);
                        }

                        leaf_body += "`";
                    }

                    leaf_body += &format!("\n- {description}");

                    if github_friendly {
                        leaf_body += &format!("\n");
                    }
                }
                _ => ()
            }
        }

        if subsection_count > 0 && leaf_body.len() != 0 {
            body += &format!("\n{} Other\n", "#".repeat(level + 1));
        }
        body += &leaf_body;
        
        body
    }

    #[async_recursion]
    pub async fn call(
        &mut self,
        backend_handles: BackendHandles,
        ctx: serenity::client::Context,
        msg: Message,
        mut args: VecDeque<String>
    ) -> Result<()> {
        match &mut self.inner {
            R6RSCommandType::RootCommand(R6RSRootCommand{ commands, section_title: _}) => {
                let next_command = args
                    .pop_front()
                    .ok_or(anyhow!("Missing subcommand!"))?;

                if next_command == "help" || next_command == ">>help" {
                    let mut body = self.description.to_owned() + "\n";
                    
                    body.push_str(&self.print_help(String::new(), 1, false).await);

                    send_embed_no_return(
                        ctx, 
                        msg, 
                        "Command Help", 
                        &body, 
                        get_random_anime_girl()
                    ).await.unwrap();

                    return Ok(());
                }

                if !commands.contains_key(&next_command) {
                    bail!("Invalid subcommand!");
                }

                commands.get_mut(&next_command)
                    .expect("Unreachable!")
                    .call(backend_handles, ctx, msg, args).await?;

                println!("Root command!");
                Ok(())
            },
            R6RSCommandType::LeafCommand(R6RSLeafCommand{function, required_authorization, valid_args: _}) => {
                // Verify that the sender of the message is in the required section
                if let Some(required_section) = required_authorization {
                    if !backend_handles.state.lock().await
                        .bot_data
                        .get("whitelisted_user_ids").ok_or(anyhow!("Missing whitelisted IDs JSON value!"))?
                        .get(&*required_section).ok_or(anyhow!("Missing that section's JSON value!"))?
                        .as_array().ok_or(anyhow!("That section isn't an array!"))?
                        .iter()
                        .any(|val| val.as_i64().expect("Unreachable") == msg.author.id.get() as i64) {
                        no_access(ctx, msg.clone(), &msg.content, msg.author.id.get()).await;
                        
                        return Ok(());
                    }
                }
                
                function.run(backend_handles, ctx, msg, args).await
                    .map_err(|e| anyhow!("Encountered an error!\n\n{e:#?}"))
            }
        }
    }
}

pub async fn inject_documentation(
    body: &str
) -> Result<()> {
    // Load the documentation template from `assets/README_TEMPLATE.md`
    let template = read_to_string("assets/README_TEMPLATE.md")
        .context("Failed to read `assets/README_TEMPLATE.md`! Does the file exist?")?;

    // Inject the body into the template
    let inject_marker = "<!-- INJECT MARKER -->";
    let injected = template.replace(inject_marker, body);

    // Write the injected template to `README.md`
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("README.md")
        .context("Failed to open file handle to `README.md`! Does the file exist?")?
        .write_all(injected.as_bytes())
        .context("Failed to write to `README.md`! Is the file in use?")?;

    println!("[ Succesfully injected documentation! :3 ]");

    Ok(())
}
pub async fn save( state: Arc<Mutex<State>> ) {
    let bot_data_serialized = &state
        .lock().await
        .bot_data
        .to_string();

    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("assets/bot_data.json")
        .expect("Failed to open file handle to `assets/bot_data.json`! Does the file exist?")
        .write_all(bot_data_serialized.as_bytes())
        .expect("Failed to write to `assets/bot_data.json`! Is the file in use?");

    println!("[ Succesfully saved! :3 ]");
}
pub async fn autosave( state: Arc<Mutex<State>> ) {
    loop {
        save( state.clone() ).await;

        sleep(Duration::from_secs(120)).await;
    }
}
pub async fn autopull( state: Arc<Mutex<State>> ) {
    loop {
        println!("[ Pulled market data :3 ]");

        let market_data_contents: String = read_to_string("assets/data.json")
            .expect("Could not find 'assets/data.json', please ensure you have created one!");
    
        state.lock()
            .await
            .market_data = serde_json::from_str(&market_data_contents)
                .expect("Could not parse the contents of 'data.json'!");

        sleep(Duration::from_secs(60)).await;
    }
}
pub async fn send_embed_no_return(
    ctx: serenity::client::Context,
    msg: Message,
    title: &str,
    description: &str,
    url: &str
) -> Result<()> {
    let embed = CreateEmbed::new()
        .title(title)
        .description(description)
        .color(get_random_color())
        .thumbnail(url);
    
    let builder = CreateMessage::new().embed(embed);
    
    tokio::spawn(msg.channel_id.send_message(ctx.http, builder));

    Ok(())
}
pub async fn send_embed(
    ctx: &serenity::client::Context,
    msg: &Message,
    title: &str,
    description: &str,
    url: &str
) -> Result<Message, String> {
    let embed = CreateEmbed::new()
        .title(title)
        .description(description)
        .color(get_random_color())
        .thumbnail(url);
    
    let builder = CreateMessage::new().embed(embed);

    msg.channel_id.send_message(&ctx.http, builder).await.map_err(|e| format!("{e:?}"))
        .map_err(|_| String::from("Failed to send error!s"))
}
pub async fn edit_embed(
    ctx: &serenity::client::Context,
    msg: &mut Message,
    title: &str,
    description: &str,
    url: &str
) {
    let embed_builder = CreateEmbed::new()
        .title(title)
        .description(description)
        .color(get_random_color())
        .thumbnail(url);
    let edit_builder = EditMessage::new().embed(embed_builder);

    msg.edit(ctx, edit_builder).await.unwrap();
}
pub fn get_random_color () -> Colour {
    vec!(
        Colour::FABLED_PINK,
        Colour::MEIBE_PINK,
        Colour::DARK_MAGENTA,
        Colour::MAGENTA
    ).choose(&mut rand::thread_rng()).unwrap().clone()
}
pub async fn _unimplemented(
    ctx: serenity::client::Context,
    msg: Message,
    cmd: &str
) {
    send_embed(
        &ctx, 
        &msg, 
        "Not yet implemented!", 
        &format!("The command **{cmd}** exists but is not yet implemented! While I work, stay cozy :3"), 
        get_random_anime_girl()
    ).await
        .unwrap();
}
pub async fn no_access(
    ctx: serenity::client::Context,
    msg: Message,
    cmd: &str,
    id: u64
) {
    send_embed(
        &ctx, 
        &msg, 
        "You don't have access to this command!", 
        &format!("You (**@{id}**) aren't authorized to use **{cmd}**.\n\n*Contact @hiibolt to purchase access or if this is in error.*"), 
        get_random_anime_girl()
    ).await
        .unwrap();
}
pub fn get_random_anime_girl() -> &'static str {
    [
        "https://github.com/hiibolt/hiibolt/assets/91273156/4a7c1e36-bf24-4f5a-a501-4dc9c92514c4",
        "https://github.com/hiibolt/hiibolt/assets/91273156/831e2922-cdcb-409d-a919-1a72fbe56ff4",
        "https://github.com/hiibolt/hiibolt/assets/91273156/9098eb3f-d883-4a8b-8c6b-525869eac2a2",
        "https://github.com/hiibolt/hiibolt/assets/91273156/d8891401-df14-435b-89a5-c23da4c38354",
        "https://github.com/hiibolt/hiibolt/assets/91273156/353dea2e-f436-4289-9a10-37f9a23e3ee6",
        "https://github.com/hiibolt/hiibolt/assets/91273156/b3cf1ffd-874b-403c-9716-dce4d4f03ae0"
    ].choose(&mut rand::thread_rng()).expect("Unreachable!")
}