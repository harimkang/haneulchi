#!/usr/bin/env bash
set -euo pipefail

swift test --package-path apps/macos
swift build --package-path apps/macos

cat evidence/notes/WF-00-WF-10-shell-navigation-checklist.md
