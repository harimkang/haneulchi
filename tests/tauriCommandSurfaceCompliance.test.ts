import { readdirSync, readFileSync, statSync } from "node:fs";
import { relative, resolve } from "node:path";
import { describe, expect, it } from "vitest";

const workspaceRoot = resolve(__dirname, "..");
const libRsPath = resolve(workspaceRoot, "src-tauri/src/lib.rs");
const frontendRoots = [resolve(workspaceRoot, "src/services"), resolve(workspaceRoot, "src/App.tsx")];

function readLibRs(): string {
  return readFileSync(libRsPath, "utf8");
}

function collectFrontendFiles(path: string): string[] {
  const stats = statSync(path);
  if (!stats.isDirectory()) return [path];

  return readdirSync(path).flatMap((entry) => {
    const child = resolve(path, entry);
    const childStats = statSync(child);
    if (childStats.isDirectory()) return collectFrontendFiles(child);
    if (!/\.(ts|tsx)$/.test(child)) return [];
    if (/\.(test|spec)\.(ts|tsx)$/.test(child)) return [];
    return [child];
  });
}

function registeredTauriCommands(source: string): Set<string> {
  const handlerBody = source.match(/tauri::generate_handler!\[([\s\S]*?)\]\)/)?.[1] ?? "";
  return new Set(
    handlerBody
      .split(",")
      .map((command) => command.trim())
      .filter(Boolean),
  );
}

function definedTauriCommands(source: string): string[] {
  return [...source.matchAll(/#\[tauri::command\]\s+fn\s+([a-z0-9_]+)/g)].map(([, command]) => command);
}

function invokedFrontendCommands(): string[] {
  return frontendRoots.flatMap((root) =>
    collectFrontendFiles(root).flatMap((path) => {
      const source = readFileSync(path, "utf8");
      return [...source.matchAll(/\binvoke(?:<[^>]+>)?\(\s*"([a-z0-9_]+)"/g)].map(
        ([, command]) => `${command} (${relative(workspaceRoot, path)})`,
      );
    }),
  );
}

describe("Tauri command surface compliance", () => {
  it("registers every Rust Tauri command in the invoke handler", () => {
    const source = readLibRs();
    const registered = registeredTauriCommands(source);
    const missing = definedTauriCommands(source).filter((command) => !registered.has(command));

    expect(missing).toEqual([]);
  });

  it("registers every frontend Tauri invoke command", () => {
    const registered = registeredTauriCommands(readLibRs());
    const missing = invokedFrontendCommands().filter((entry) => {
      const command = entry.match(/^([a-z0-9_]+)/)?.[1];
      return !command || !registered.has(command);
    });

    expect(missing).toEqual([]);
  });
});
