import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { relative, resolve } from "node:path";
import { describe, expect, it } from "vitest";

const workspaceRoot = resolve(__dirname, "..");
const srcRoot = resolve(workspaceRoot, "src");
const hexColorPattern = /#[0-9a-fA-F]{3,8}\b/g;
const sourceFilePattern = /\.(css|ts|tsx)$/;
const testFilePattern = /\.(test|spec)\.(ts|tsx)$/;
const packageJson = JSON.parse(readFileSync(resolve(workspaceRoot, "package.json"), "utf8")) as {
  scripts: Record<string, string>;
};
const designSystemTokenSourcePath = resolve(
  workspaceRoot,
  "docs/haneulchi_frontend_design_system_v2/haneulchi-ui-tokens-v2.css",
);
const includePrivateDocs =
  (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env
    ?.HANEULCHI_INCLUDE_PRIVATE_DOCS === "1";
const itWithDesignDocs = includePrivateDocs && existsSync(designSystemTokenSourcePath) ? it : it.skip;
const allowedTokenFiles = new Set([
  "src/design/haneulchiDesignTokens.ts",
  "src/design/haneulchi-layout-constants-v2.ts",
  "src/design/haneulchi-screen-layout-presets-v2.ts",
  "src/styles/haneulchi-ui-tokens-v2.css",
]);

function collectSourceFiles(directory: string): string[] {
  return readdirSync(directory).flatMap((entry) => {
    const path = resolve(directory, entry);
    const stats = statSync(path);
    if (stats.isDirectory()) return collectSourceFiles(path);
    if (!sourceFilePattern.test(path)) return [];
    return [path];
  });
}

function readSource(relativePath: string): string {
  return readFileSync(resolve(workspaceRoot, relativePath), "utf8");
}

describe("design token compliance", () => {
  it("keeps raw hex colors inside design token files", () => {
    const offenders = collectSourceFiles(srcRoot).flatMap((path) => {
      const relativePath = relative(workspaceRoot, path);
      if (testFilePattern.test(relativePath)) return [];
      if (allowedTokenFiles.has(relativePath)) return [];
      const matches = readFileSync(path, "utf8").match(hexColorPattern) ?? [];
      return matches.map((color) => `${relativePath}: ${color}`);
    });

    expect(offenders).toEqual([]);
  });

  it("keeps the right rail as a compact drawer in the documented 1100-1279px range", () => {
    const appCss = readSource("src/App.css");

    expect(appCss).toContain("@media (max-width: 1279px)");
    expect(appCss).toContain(".hc-right-rail.is-open");
    expect(appCss).toContain("transform: translateX(100%)");
    expect(appCss).toContain("transform: translateX(0)");
  });

  it("keeps the documented v2 design implementation files present in src", () => {
    [
      "src/styles/haneulchi-ui-tokens-v2.css",
      "src/design/haneulchi-tailwind-preset-v2.ts",
      "src/design/haneulchi-layout-constants-v2.ts",
      "src/design/haneulchi-screen-layout-presets-v2.ts",
    ].forEach((relativePath) => {
      expect(existsSync(resolve(workspaceRoot, relativePath)), relativePath).toBe(true);
    });
  });

  itWithDesignDocs("keeps runtime CSS tokens synchronized with the design-system token source", () => {
    expect(readSource("src/styles/haneulchi-ui-tokens-v2.css")).toBe(
      readFileSync(designSystemTokenSourcePath, "utf8"),
    );
  });

  it("exposes an executable visual QA route contract command", () => {
    expect(packageJson.scripts["test:visual:qa"]).toBe(
      'vitest run src/App.test.tsx --testNamePattern "visual QA route|dynamic visual QA|concept asset filename aliases"',
    );
  });
});
