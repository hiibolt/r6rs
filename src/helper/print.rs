use colored::Colorize;

pub fn startup(
    message: &str
) {
    let to_print = format!("[Startup] {}", message);

    println!("{}", to_print.green());
}
#[macro_export]
macro_rules! info{
    ($a:expr) => {
        let message = format!($a);
        let to_print = format!("[Info] {}", message);

        println!("{}", to_print.blue());
    }
}
#[macro_export]
macro_rules! warn{
    ($a:expr) => {
        let message = format!($a);
        let to_print = format!("[!] {}", message);

        println!("{}", to_print.yellow());
    }
}
#[macro_export]
macro_rules! error{
    ($a:expr) => {
        let message = format!($a);
        let to_print = format!("[X] {}", message);

        println!("{}", to_print.red());
    }
}
#[macro_export]
macro_rules! daemon{
    ($a:expr) => {
        let message = format!($a);
        let to_print = format!("[Daemon] {}", message);

        println!("{}", to_print.purple());
    }
}