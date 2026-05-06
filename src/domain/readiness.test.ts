import { describe, expect, it } from "vitest";
import { getReadinessSummary, readinessChecks } from "./readiness";

describe("readiness model", () => {
  it("summarizes ready, warning, and missing checks", () => {
    const summary = getReadinessSummary(readinessChecks);

    expect(summary.ready).toBeGreaterThan(0);
    expect(summary.warning).toBeGreaterThan(0);
    expect(summary.missing).toBeGreaterThan(0);
    expect(summary.total).toBe(readinessChecks.length);
  });

  it("keeps the generic shell path available when agent CLIs are missing", () => {
    expect(readinessChecks).toEqual(
      expect.arrayContaining([
        expect.objectContaining({
          id: "generic-shell",
          status: "ready",
          label: "Generic Shell fallback",
        }),
      ]),
    );
  });
});
