const assert = require("assert");

async function run() {
  const mod = await import("./dist/main.mjs");

  assert.strictEqual(typeof mod.Buffer, "function");
  assert.strictEqual(typeof mod.Blob, "function");
  assert.strictEqual(typeof mod.File, "function");
  assert.strictEqual(typeof mod.atob, "function");
  assert.strictEqual(typeof mod.btoa, "function");
  assert.strictEqual(typeof mod.isAscii, "function");
  assert.strictEqual(typeof mod.isUtf8, "function");
  assert.strictEqual(typeof mod.transcode, "function");

  const buf = mod.Buffer.from("hello");
  assert.strictEqual(buf.toString(), "hello");

  const blob = new mod.Blob(["test"]);
  assert.strictEqual(await blob.text(), "test");

  console.log("CommonJS integration test passed.");
}

run().catch((err) => {
  console.error(err);
  process.exit(1);
});
