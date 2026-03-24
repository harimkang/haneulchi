fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let result = hc_cli::run(&args);

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
