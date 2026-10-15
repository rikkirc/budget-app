import test from "node:test";
import assert from "node:assert/strict";
import { toBase64, fromBase64 } from "./base64.js";

test("vault bytes survive the trip through base64, including multi-MB vaults", () => {
  const bytes = new Uint8Array(3_000_000).map((_, i) => (i * 7919) % 256);
  const text = toBase64(bytes);
  assert.equal(typeof text, "string");
  assert.deepEqual(fromBase64(text), bytes);
  assert.equal(Buffer.from(bytes.subarray(0, 1000)).toString("base64"), toBase64(bytes.subarray(0, 1000)));
});
