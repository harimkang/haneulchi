use std::fs;
use std::io;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/terminal")
}

fn fixture_path(name: &str) -> io::Result<PathBuf> {
    match name {
        "hello-world.ansi" | "alternate-screen.ansi" => Ok(fixture_root().join(name)),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unknown terminal transcript fixture",
        )),
    }
}

pub fn load_fixture(name: &str) -> io::Result<String> {
    fs::read_to_string(fixture_path(name)?)
}
