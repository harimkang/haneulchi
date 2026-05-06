import { describe, expect, it } from "vitest";
import packageJson from "../../package.json";

const rawFiles = import.meta.glob("../../{.github,docs,packaging,public,scripts}/**/*", {
  eager: true,
  query: "?raw",
  import: "default",
}) as Record<string, string>;
const includePrivateDocs =
  (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env
    ?.HANEULCHI_INCLUDE_PRIVATE_DOCS === "1";
const releaseRunbookDocsAvailable =
  includePrivateDocs &&
  Boolean(rawFiles["../../docs/release/macos-distribution.md"]) &&
  Boolean(rawFiles["../../docs/release/update-feed.md"]);
const itWithReleaseRunbookDocs = releaseRunbookDocsAvailable ? it : it.skip;

function raw(relativePath: string): string {
  const content = rawFiles[`../../${relativePath}`];
  expect(content, `${relativePath} should exist`).toBeDefined();
  return content;
}

describe("macOS release packaging pipeline", () => {
  it("defines repeatable DMG build, signing, notarization, and verification entrypoints", () => {
    const verifier = raw("scripts/release/verify-macos-artifacts.sh");
    const notarizer = raw("scripts/release/notarize-macos.sh");

    expect(packageJson.scripts["release:macos:app"]).toBe("tauri build --bundles app");
    expect(packageJson.scripts["release:macos:dmg"]).toBe("tauri build --bundles dmg");
    expect(packageJson.scripts["release:macos:verify"]).toContain("scripts/release/verify-macos-artifacts.sh");
    expect(packageJson.scripts["release:macos:notarize"]).toContain("scripts/release/notarize-macos.sh");

    expect(verifier).toContain("#!/usr/bin/env bash");
    expect(verifier).toContain("codesign");
    expect(verifier).toContain("spctl");
    expect(verifier).toContain("hdiutil attach");
    expect(verifier).toContain("verify_app_signature");
    expect(verifier).toContain("MOUNTED_APP_PATH");
    expect(verifier).toContain("notarization");

    expect(notarizer).toContain("#!/usr/bin/env bash");
    expect(notarizer).toContain("xcrun notarytool submit");
    expect(notarizer).toContain("xcrun stapler staple");
  });

  itWithReleaseRunbookDocs("documents and wires GitHub Actions secrets for signed/notarized DMG release builds", () => {
    const workflow = raw(".github/workflows/release-macos.yml");
    const runbook = raw("docs/release/macos-distribution.md");

    expect(workflow).toContain("tauri build --bundles dmg");
    expect(workflow).toContain("release:macos:app");
    expect(workflow).toContain("APPLE_CERTIFICATE");
    expect(workflow).toContain("APPLE_CERTIFICATE_PASSWORD");
    expect(workflow).toContain("APPLE_SIGNING_IDENTITY");
    expect(workflow).toContain("APPLE_ID");
    expect(workflow).toContain("APPLE_PASSWORD");
    expect(workflow).toContain("APPLE_TEAM_ID");
    expect(workflow).toContain("actions/upload-artifact");

    expect(runbook).toContain("APPLE_CERTIFICATE");
    expect(runbook).toContain("notarytool");
    expect(runbook).toContain("release:macos:verify");
  });

  itWithReleaseRunbookDocs("defines update feed channels and a verifier for release-channel settings", () => {
    const feedVerifier = raw("scripts/release/verify-update-feed.sh");
    const stableFeed = raw("public/update-feed/stable.json");
    const betaFeed = raw("public/update-feed/beta.json");
    const updateFeedRunbook = raw("docs/release/update-feed.md");

    expect(packageJson.scripts["release:update-feed:verify"]).toContain("scripts/release/verify-update-feed.sh");
    expect(feedVerifier).toContain("stable.json");
    expect(feedVerifier).toContain("beta.json");
    expect(feedVerifier).toContain("darwin-aarch64");

    const stable = JSON.parse(stableFeed) as { version: string; platforms: Record<string, unknown> };
    const beta = JSON.parse(betaFeed) as { version: string; platforms: Record<string, unknown> };
    expect(stable.version).toBe(packageJson.version);
    expect(beta.version).toBe(packageJson.version);
    expect(stable.platforms["darwin-aarch64"]).toBeTruthy();
    expect(beta.platforms["darwin-aarch64"]).toBeTruthy();
    expect(updateFeedRunbook).toContain("release:update-feed:verify");
    expect(updateFeedRunbook).toContain("stable");
    expect(updateFeedRunbook).toContain("beta");
  });

  it("wires Homebrew cask publishing artifacts without embedding secrets", () => {
    const caskRenderer = raw("scripts/release/render-homebrew-cask.sh");
    const caskTemplate = raw("packaging/homebrew/haneulchi.rb.template");
    const caskWorkflow = raw(".github/workflows/homebrew-cask.yml");

    expect(packageJson.scripts["release:homebrew:cask"]).toContain("scripts/release/render-homebrew-cask.sh");
    expect(caskRenderer).toContain("HOMEBREW_TAP_REPOSITORY");
    expect(caskRenderer).toContain("sha256");
    expect(caskTemplate).toContain("cask \"haneulchi\"");
    expect(caskTemplate).toContain("__DMG_SHA256__");
    expect(caskWorkflow).toContain("HOMEBREW_TAP_TOKEN");
    expect(caskWorkflow).toContain("release:homebrew:cask");
  });

  it("wires crash symbol upload artifacts with explicit provider secrets", () => {
    const symbolUploader = raw("scripts/release/upload-symbols.sh");
    const crashWorkflow = raw(".github/workflows/upload-symbols.yml");

    expect(packageJson.scripts["release:symbols:upload"]).toContain("scripts/release/upload-symbols.sh");
    expect(symbolUploader).toContain("SENTRY_AUTH_TOKEN");
    expect(symbolUploader).toContain("SENTRY_ORG");
    expect(symbolUploader).toContain("SENTRY_PROJECT");
    expect(symbolUploader).toContain("sentry-cli");
    expect(crashWorkflow).toContain("SENTRY_AUTH_TOKEN");
    expect(crashWorkflow).toContain("release:symbols:upload");
  });
});
