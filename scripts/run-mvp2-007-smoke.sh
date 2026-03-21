#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

swift test --package-path "${repo_root}/apps/macos" --filter 'DemoWorkspaceScaffoldTests|WelcomeReadinessViewModelTests|AppShellBootstrapTests'
swift build --package-path "${repo_root}/apps/macos"
