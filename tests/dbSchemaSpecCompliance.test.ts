import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";

const workspaceRoot = resolve(__dirname, "..");
const dbSchemaSpecPath = resolve(
  workspaceRoot,
  "docs/Haneulchi_Document_Set_v10_Super_App_Pack/04_specs/Haneulchi_DB_Schema_Spec_v4.md",
);
const stateStorePath = resolve(workspaceRoot, "src-tauri/src/state_store.rs");
const includePrivateDocs =
  (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env
    ?.HANEULCHI_INCLUDE_PRIVATE_DOCS === "1";
const describeWithDocs = includePrivateDocs && existsSync(dbSchemaSpecPath) ? describe : describe.skip;

type TableColumns = Record<string, string[]>;

function readDbSchemaSpec(): string {
  return readFileSync(dbSchemaSpecPath, "utf8");
}

function readStateStore(): string {
  return readFileSync(stateStorePath, "utf8");
}

function implementedTables(source: string): TableColumns {
  const tables: TableColumns = {};
  const tablePattern =
    /CREATE TABLE IF NOT EXISTS\s+([a-z0-9_]+)\(([\s\S]*?)\n\s*\);/g;

  for (const [, tableName, body] of source.matchAll(tablePattern)) {
    tables[tableName] = body
      .split("\n")
      .map((line) => line.trim().replace(/,$/, ""))
      .filter(Boolean)
      .filter((line) => !/^(PRIMARY KEY|UNIQUE|FOREIGN KEY|CHECK)\b/.test(line))
      .map((line) => line.split(/\s+/)[0]);
  }

  return tables;
}

function documentedTables(spec: string): TableColumns {
  const tables: TableColumns = {};
  const coreTablesSection =
    spec.match(/## 2\. Core tables([\s\S]*?)## 3\. Required indexes/)?.[1] ?? "";
  const tablePattern = /^([a-z0-9_]+)\(([^)]*)\)/gm;

  for (const [, tableName, columnList] of coreTablesSection.matchAll(tablePattern)) {
    tables[tableName] = columnList.split(",").map((column) => column.trim());
  }

  return tables;
}

function implementedIndexes(source: string): string[] {
  return [...source.matchAll(/CREATE INDEX IF NOT EXISTS\s+([a-z0-9_]+)\s+ON\s+([a-z0-9_]+)\(([^)]*)\);/g)].map(
    ([, indexName, tableName, columns]) => `${indexName}: ${tableName}(${columns})`,
  );
}

describeWithDocs("DB schema spec compliance", () => {
  it("documents every SQLite table and column created by the state store", () => {
    const implemented = implementedTables(readStateStore());
    const documented = documentedTables(readDbSchemaSpec());

    const missing = Object.entries(implemented).flatMap(([tableName, columns]) => {
      const documentedColumns = documented[tableName];
      if (!documentedColumns) return [`${tableName}: <table missing>`];

      return columns
        .filter((column) => !documentedColumns.includes(column))
        .map((column) => `${tableName}.${column}`);
    });

    expect(missing).toEqual([]);
  });

  it("documents every SQLite index created by the state store", () => {
    const spec = readDbSchemaSpec();
    const missing = implementedIndexes(readStateStore()).filter(
      (index) => !spec.includes(`\`${index}\``),
    );

    expect(missing).toEqual([]);
  });
});
