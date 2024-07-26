use super::command::R6RSCommand;

pub async fn build_root_command() -> R6RSCommand {
    let admin_commands   = crate::sections::admin::build_admin_commands().await;
    let econ_commands    = crate::sections::econ::build_econ_commands().await;
    let osint_commands   = crate::sections::osint::build_osint_commands().await;
    let opsec_commands   = crate::sections::opsec::build_opsec_commands().await;
    let mut root_command = R6RSCommand::new_root(
        String::from("R6RS is a general purpose bot, orignally intended for Rainbow Six Siege, but since multipurposed into a powerful general OSINT tool."),
        String::from("Commands")
    );
    let mut r6_root_command = R6RSCommand::new_root(
        String::from("Commands specifically related to R6."),
        String::from("R6")
    );

    r6_root_command.attach(
        String::from("econ"),
        econ_commands
    );
    r6_root_command.attach(
        String::from("opsec"),
        opsec_commands
    );
    root_command.attach(
        String::from(">>r6"),
        r6_root_command
    );
    root_command.attach(
        String::from(">>admin"),
        admin_commands
    );
    root_command.attach(
        String::from(">>osint"),
        osint_commands
    );

    root_command
}