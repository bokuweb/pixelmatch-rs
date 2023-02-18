import assert from "node:assert";
import test from "node:test";

import { pixelmatch } from "../../node/esm/index.mjs";

test("it passes", () => {
  const res = pixelmatch(
    new Uint8Array([255, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    new Uint8Array([0, 0, 0, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
    2,
    2,
    {
      includeAntiAlias: true,
      threshold: 0.2,
    }
  );
  assert.equal(res.count, 1);
  assert.deepEqual(
    res.diff,
    new Uint8Array([
      255, 119, 119, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
      255,
    ])
  );
});
