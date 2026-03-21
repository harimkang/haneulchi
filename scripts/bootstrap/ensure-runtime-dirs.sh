#!/usr/bin/env bash
set -euo pipefail

base_dir="${HOME}/Library/Application Support/Haneulchi"

mkdir -p "${base_dir}/run"
mkdir -p "${base_dir}/workspaces"
mkdir -p "${base_dir}/artifacts"
mkdir -p "${base_dir}/db"

printf 'Ensured runtime directories under %s\n' "${base_dir}"
