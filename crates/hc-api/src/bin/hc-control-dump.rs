fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    hc_control_plane::reset_task_board_for_tests();
    hc_control_plane::reset_review_queue_for_tests();

    let output = match args.first().map(String::as_str) {
        Some("state") => hc_api::state::state_json(),
        Some("sessions") => hc_api::sessions::sessions_list_json(),
        Some("review-queue") => hc_api::review_queue_json(),
        _ => Err("unsupported dump command".to_string()),
    }
    .expect("dump output");

    println!("{output}");
}
