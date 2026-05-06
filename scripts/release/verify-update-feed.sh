#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PACKAGE_JSON="$ROOT_DIR/package.json"
FEED_DIR="${FEED_DIR:-$ROOT_DIR/public/update-feed}"
REQUIRE_RELEASE_SIGNATURES="${REQUIRE_RELEASE_SIGNATURES:-0}"

node --input-type=module - "$PACKAGE_JSON" "$FEED_DIR" "$REQUIRE_RELEASE_SIGNATURES" <<'NODE'
import { readFileSync } from "node:fs";
import { join } from "node:path";

const [, , packagePath, feedDir, requireReleaseSignatures] = process.argv;
const packageJson = JSON.parse(readFileSync(packagePath, "utf8"));
const channels = ["stable.json", "beta.json"];
const requiredPlatforms = ["darwin-aarch64", "darwin-x86_64"];

for (const channel of channels) {
  const feedPath = join(feedDir, channel);
  const feed = JSON.parse(readFileSync(feedPath, "utf8"));
  if (feed.version !== packageJson.version) {
    throw new Error(`${channel} version ${feed.version} does not match package.json ${packageJson.version}`);
  }
  if (!feed.pub_date || Number.isNaN(Date.parse(feed.pub_date))) {
    throw new Error(`${channel} must include an ISO pub_date`);
  }
  for (const platform of requiredPlatforms) {
    const entry = feed.platforms?.[platform];
    if (!entry) {
      throw new Error(`${channel} missing ${platform}`);
    }
    if (!String(entry.url ?? "").startsWith("https://")) {
      throw new Error(`${channel} ${platform} url must be https`);
    }
    if (!String(entry.signature ?? "").trim()) {
      throw new Error(`${channel} ${platform} signature is required`);
    }
    if (requireReleaseSignatures === "1" && String(entry.signature).includes("SIGNATURE_REQUIRED")) {
      throw new Error(`${channel} ${platform} still has a placeholder release signature`);
    }
  }
}

console.log("update feed channels verified");
NODE
