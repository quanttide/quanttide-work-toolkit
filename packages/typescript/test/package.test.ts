import { readFileSync } from "node:fs";

import { describe, expect, it } from "vitest";

import { DOMAIN, VERSION } from "../src/index.js";

const pkg = JSON.parse(
  readFileSync(new URL("../package.json", import.meta.url), "utf-8"),
) as { version: string };

describe("quanttide-work 包元数据", () => {
  it("领域英文名", () => {
    expect(DOMAIN).toBe("knowledge-work");
  });

  it("包版本与 package.json 一致", () => {
    expect(VERSION).toBe(pkg.version);
  });
});
