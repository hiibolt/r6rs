use super::{bot::Sendable, lib::{get_random_anime_girl, AsyncFnPtr}};
use crate::helper::bot::BackendHandles;

use std::{collections::{HashMap, VecDeque}, sync::Arc};

use anyhow::{Result, anyhow, bail};
use async_recursion::async_recursion;
use serenity::all::{CreateCommand, CreateCommandOption};
use tokio::sync::Mutex;


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
    pub async fn build_commands(
        &mut self,
        prefix: String,
    ) -> Vec<CreateCommand> {
        let mut ret = Vec::new();
        
        let R6RSRootCommand{ commands, section_title: _ } = if let R6RSCommandType::RootCommand(root_command) = &mut self.inner {
            root_command
        } else {
            panic!("Cannot build commands on a leaf command!");
        };

        // Iterate through each command
        for (name, command) in commands.iter_mut() {
            match &command.inner {
                R6RSCommandType::RootCommand(_) => {
                    let nested_commands = command.build_commands(prefix.clone() + "-" + &name).await;
                    ret.extend(nested_commands);
                },
                R6RSCommandType::LeafCommand(R6RSLeafCommand{required_authorization: _, valid_args, function: _}) => {
                    let options: Vec<String> = valid_args.iter().max_by_key(|set| set.len())
                        .expect("Leaf commands must have at least one valid argument set!")
                        .iter()
                        .map(|arg| {
                            arg.replace(" ", "-")
                                .replace("$", "")
                                .replace("|", "or")
                                .replace("#", "num")
                        })
                        .collect();
                    let required = valid_args.len() == 1;

                    let command_name = format!("{}-{}", prefix, name).replace("->>", "");
                    let description: String = String::from("Run >>help to see a description!");
                    let mut command = CreateCommand::new(&command_name)
                        .description(&description);

                    for option in &options {
                        if option == "file" {
                            command = command.add_option(
                                CreateCommandOption::new(
                                    serenity::all::CommandOptionType::Attachment,
                                    option,
                                    "File"
                                )
                                .required(required));

                            continue;
                        }

                        command = command.add_option(
                            CreateCommandOption::new(
                                serenity::all::CommandOptionType::String,
                                option,
                                "Argument"
                            )
                            .required(required));
                    }

                    ret.push(command);
                }
            }
        }

        ret
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
        sendable: Arc<Mutex<Sendable>>,
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

                    sendable.lock().await.send(
                        "Command Help".to_string(),
                        body,
                        get_random_anime_girl().to_string()
                    ).await.expect("Failed to send help message!");

                    sendable.lock().await.finalize()
                        .await.expect("Failed to finalize message!");

                    return Ok(());
                }

                if !commands.contains_key(&next_command) {
                    bail!("Invalid subcommand!\n\nRun >>help to see a list of commands!");
                }

                commands.get_mut(&next_command)
                    .expect("Unreachable!")
                    .call(backend_handles, sendable, args).await?;
                Ok(())
            },
            R6RSCommandType::LeafCommand(R6RSLeafCommand{function, required_authorization, valid_args: _}) => {
                // This only applies to Discord sendables
                let value = sendable.lock().await;
                if let Sendable::DiscordResponseSender(ref inner) = *value {
                    // Verify that the sender of the message is in the required section
                    if let Some(required_section) = required_authorization {
                        if !backend_handles.state.lock().await
                            .bot_data
                            .get("whitelisted_user_ids").ok_or(anyhow!("Missing whitelisted IDs JSON value!"))?
                            .get(&*required_section).ok_or(anyhow!("Missing that section's JSON value!"))?
                            .as_array().ok_or(anyhow!("That section isn't an array!"))?
                            .iter()
                            .any(|val| val.as_i64().expect("Unreachable") == inner.author.id.get() as i64) {
                            
                            sendable.lock().await.send(
                                "No Access".to_string(),
                                "You do not have access to this command!".to_string(),
                                get_random_anime_girl().to_string()
                            ).await
                                .unwrap();
                            
                            return Ok(());
                        }
                    }
                }
                
                function.run(backend_handles, sendable.clone(), args).await
                    .map_err(|e| anyhow!("Encountered an error!\n\n{e:#?}"))
            }
        }
    }
}