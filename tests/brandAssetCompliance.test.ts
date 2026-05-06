import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const workspaceRoot = resolve(__dirname, "..");

function readSource(relativePath: string): string {
  return readFileSync(resolve(workspaceRoot, relativePath), "utf8");
}

describe("brand asset compliance", () => {
  it("links Haneulchi favicon assets from the app HTML entry point", () => {
    const html = readSource("index.html");

    expect(html).toContain('<link rel="icon" href="/favicon.ico" sizes="any" />');
    expect(html).toContain('<link rel="icon" type="image/png" sizes="32x32" href="/favicon-32x32.png" />');
    expect(html).toContain('<link rel="apple-touch-icon" sizes="180x180" href="/apple-touch-icon.png" />');
    expect(html).toContain('<link rel="manifest" href="/site.webmanifest" />');
  });

  it("keeps web favicon files present in public assets", () => {
    [
      "public/favicon.ico",
      "public/favicon-16x16.png",
      "public/favicon-32x32.png",
      "public/apple-touch-icon.png",
      "public/android-chrome-192x192.png",
      "public/android-chrome-512x512.png",
      "public/site.webmanifest",
    ].forEach((relativePath) => {
      expect(existsSync(resolve(workspaceRoot, relativePath)), relativePath).toBe(true);
    });
  });

  it("uses the Haneulchi dark app background for Android adaptive icons", () => {
    expect(readSource("src-tauri/icons/android/values/ic_launcher_background.xml")).toContain(
      '<color name="ic_launcher_background">#081014</color>',
    );
  });
});
