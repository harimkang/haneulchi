pub mod client;
pub mod dispatch;
pub mod output;
pub mod reconcile;
pub mod session;
pub mod state;
pub mod task;
pub mod workflow;

pub fn run(args: &[String]) -> Result<String, String> {
    let client = client::ControlClient::from_env()?;
    match args.first().map(String::as_str) {
        Some("state") => state::run(&client, &args[1..]),
        Some("session") => session::run(&client, &args[1..]),
        Some("task") => task::run(&client, &args[1..]),
        Some("workflow") => workflow::run(&client, &args[1..]),
        Some("reconcile") => reconcile::run(&client, &args[1..]),
        Some("dispatch") => dispatch::run(&client, &args[1..]),
        _ => Err("unsupported command".to_string()),
    }
}
