#!/bin/sh
set -eu

workspace_root="$1"
output_dir="$2"
cargo_target_dir="$output_dir/cargo-target"
fixtures_dir="$workspace_root/fixtures/terminal"
generated_swift="$output_dir/GeneratedTerminalFixtures.swift"

mkdir -p "$output_dir"
cd "$workspace_root"

export CARGO_TARGET_DIR="$cargo_target_dir"
cargo build -p hc-ffi
cp "$cargo_target_dir/debug/libhc_ffi.a" "$output_dir/libhc_ffi.a"

/usr/bin/python3 - "$fixtures_dir" "$generated_swift" <<'PY'
import base64
import pathlib
import sys

fixtures_dir = pathlib.Path(sys.argv[1])
generated_swift = pathlib.Path(sys.argv[2])

fixture_paths = sorted(fixtures_dir.glob("*.ansi"))

lines = [
    "import Foundation",
    "",
    "enum GeneratedTerminalTranscriptFixtures {",
    "    static let fixtures: [String: String] = [",
]

for fixture_path in fixture_paths:
    payload = base64.b64encode(fixture_path.read_bytes()).decode("ascii")
    lines.append(f'        "{fixture_path.name}": decode("{payload}"),')

lines.extend([
    "    ]",
    "",
    "    private static func decode(_ payload: String) -> String {",
    "        let data = Data(base64Encoded: payload)!",
    "        return String(decoding: data, as: UTF8.self)",
    "    }",
    "}",
    "",
])

generated_swift.write_text("\n".join(lines), encoding="utf-8")
PY
