pub fn run(args: &[String]) -> Result<String, String> {
    if args.iter().any(|arg| arg == "--json") {
        hc_ffi::state_snapshot_json()
    } else {
        hc_ffi::state_snapshot_json()
    }
}
