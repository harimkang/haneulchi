use hc_runtime::terminal::transcript::load_fixture;
use std::io::ErrorKind;

#[test]
fn loads_hello_world_fixture_with_escape_sequences() {
    let transcript = load_fixture("hello-world.ansi").expect("fixture should load");

    assert!(transcript.contains('\u{1b}'));
    assert!(transcript.contains("Haneulchi"));
}

#[test]
fn loads_alternate_screen_fixture_with_canary_sequences() {
    let transcript = load_fixture("alternate-screen.ansi").expect("fixture should load");

    assert!(transcript.contains("\u{1b}[?1049h"));
    assert!(transcript.contains("\u{1b}[?1049l"));
}

#[test]
fn rejects_fixture_traversal_segments() {
    let error = load_fixture("../hello-world.ansi").expect_err("traversal should be rejected");

    assert_eq!(error.kind(), ErrorKind::InvalidInput);
}

#[test]
fn rejects_unknown_fixture_names() {
    let error = load_fixture("unknown.ansi").expect_err("unknown names should be rejected");

    assert_eq!(error.kind(), ErrorKind::InvalidInput);
}
