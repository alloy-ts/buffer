/**
 * examples/limits.ts
 *
 * Demonstrates the size/limit constants and the deprecated `SlowBuffer`
 * helper exposed by `@alloy-ts/buffer`, plus the error behavior of
 * `Buffer.alloc` for invalid sizes.
 *
 * Run with oxc-node:
 *
 *   node --import @oxc-node/core/register examples/limits.ts
 */
import { Buffer, constants, INSPECT_MAX_BYTES, kMaxLength, kStringMaxLength, poolSize, SlowBuffer } from "../dist/index.js";

const log = (label: string, value: unknown): void => {
  console.log(`\x1b[36m${label}\x1b[0m`, value);
};

console.log("=== Size / limit constants ===");

log("constants.MAX_LENGTH", constants.MAX_LENGTH); // 2147483647
log("constants.MAX_STRING_LENGTH", constants.MAX_STRING_LENGTH); // 536870888
log("kMaxLength (alias)", kMaxLength); // 2147483647
log("kStringMaxLength (alias)", kStringMaxLength); // 536870888
log("INSPECT_MAX_BYTES", INSPECT_MAX_BYTES); // 50

console.log("\n=== poolSize ===");

log("poolSize", poolSize); // 65536

console.log("\n=== Deprecated SlowBuffer ===");

const slow = SlowBuffer(4);
log("SlowBuffer(4) isBuffer", Buffer.isBuffer(slow)); // true
log("SlowBuffer(4) length", slow.length); // 4
log("SlowBuffer.prototype === Buffer.prototype", (slow as unknown as { constructor: unknown }).constructor === Buffer); // true

console.log("\n=== alloc error behavior ===");

function tryAlloc(size: number): string {
  try {
    Buffer.alloc(size);
    return "ok";
  } catch (err) {
    return `${err instanceof Error ? err.constructor.name : "Error"}: ${(err as Error).message}`;
  }
}

log("alloc(-1)", tryAlloc(-1)); // RangeError
log("alloc(NaN)", tryAlloc(NaN)); // TypeError
log("alloc(1e9) ok", tryAlloc(1_000_000_000)); // ok (within kMaxLength)

console.log("\n=== isBuffer discrimination ===");

log("isBuffer(Buffer)", Buffer.isBuffer(Buffer.alloc(1))); // true
log("isBuffer(Uint8Array)", Buffer.isBuffer(new Uint8Array(1))); // false
log("isBuffer(plain object)", Buffer.isBuffer({ _isBuffer: true })); // true ( duck-typed )

console.log("\nAll limits examples executed successfully.");
