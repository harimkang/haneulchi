#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo_target_dir="${CARGO_TARGET_DIR:-${repo_root}/target}"

log() {
  printf '%s\n' "$1"
}

cd "${repo_root}"

log "Building hc-ffi static library"
cargo build -p hc-ffi

vendor_lib_dir="${repo_root}/apps/macos/Vendor/lib"
vendor_include_dir="${repo_root}/apps/macos/Vendor/HCCoreFFI/include"
vendor_header_path="${vendor_include_dir}/hc_ffi.h"
mkdir -p "${vendor_lib_dir}" "${vendor_include_dir}"

install -m 0644 "${cargo_target_dir}/debug/libhc_ffi.a" "${vendor_lib_dir}/libhc_ffi.a"
log "Synced ${cargo_target_dir}/debug/libhc_ffi.a -> apps/macos/Vendor/lib/libhc_ffi.a"

ln -sfn "../../../../../crates/hc-ffi/include/hc_ffi.h" "${vendor_header_path}"
log "Aligned apps/macos/Vendor/HCCoreFFI/include/hc_ffi.h -> crates/hc-ffi/include/hc_ffi.h"
