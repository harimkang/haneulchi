mod state;
mod task;
mod workflow;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let result = match args.first().map(String::as_str) {
        Some("state") => state::run(&args[1..]),
        Some("task") => task::run(&args[1..]),
        Some("workflow") => workflow::run(&args[1..]),
        Some("reconcile") if args.get(1).map(String::as_str) == Some("now") => {
            hc_api::reconcile_now_json()
        }
        _ => Err("unsupported command".to_string()),
    };

    match result {
        Ok(output) => {
            println!("{output}");
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
