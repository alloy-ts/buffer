/**
 * examples/binary-io.ts
 *
 * Demonstrates the numeric read/write API of `@alloy-ts/buffer`: full
 * big-endian / little-endian coverage for 8/16/32/64-bit integers and
 * BigInts, IEEE-754 floats/doubles, variable-length integers (1..6 bytes),
 * byte swapping, and a small binary-protocol round-trip.
 *
 * Run with oxc-node:
 *
 *   node --import @oxc-node/core/register examples/binary-io.ts
 */
import { Buffer } from "../src/buffer.js";

const log = (label: string, value: unknown): void => {
  console.log(`\x1b[36m${label}\x1b[0m`, value);
};

console.log("=== 8 / 16 / 32-bit integers ===");

const b = Buffer.alloc(8);
log("writeInt8(-1) -> next offset", b.writeInt8(-1, 0)); // 1
log("readInt8", b.readInt8(0)); // -1

log("writeUInt16LE(0x1234) -> offset", b.writeUInt16LE(0x1234, 0)); // 2
log("readUInt16LE", b.readUInt16LE(0).toString(16)); // "1234"
log("readUInt16BE (swapped)", b.readUInt16BE(0).toString(16)); // "3412"

log("writeInt32BE(-123456) -> offset", b.writeInt32BE(-123456, 0)); // 4
log("readInt32BE", b.readInt32BE(0)); // -123456

console.log("\n=== 64-bit BigInt ===");

const big = Buffer.alloc(8);
big.writeBigUInt64LE(18446744073709551615n, 0);
log("writeBigUInt64LE max u64", big.readBigUInt64LE(0)); // 18446744073709551615n
big.writeBigInt64BE(-9007199254740991n, 0);
log("writeBigInt64BE -> readBigInt64BE", big.readBigInt64BE(0)); // -9007199254740991n

console.log("\n=== IEEE-754 floats / doubles ===");

const f = Buffer.alloc(8);
f.writeFloatLE(1.5, 0);
log("writeFloatLE(1.5) -> readFloatLE", f.readFloatLE(0)); // 1.5
f.writeDoubleBE(3.141592653589793, 0);
log("writeDoubleBE(pi) -> readDoubleBE", f.readDoubleBE(0)); // ~3.141592653589793

console.log("\n=== Variable-length integers (1..6 bytes) ===");

const v = Buffer.alloc(6);
v.writeUIntLE(0x123456, 0, 3);
log("writeUIntLE(0x123456, 3) -> readUIntLE", v.readUIntLE(0, 3).toString(16)); // "123456"
v.writeIntLE(-1000, 0, 2);
log("writeIntLE(-1000, 2) -> readIntLE", v.readIntLE(0, 2)); // -1000
log(
  "writeUIntBE(0xabcdef, 3) -> readUIntBE",
  (() => {
    const x = Buffer.alloc(3);
    x.writeUIntBE(0xabcdef, 0, 3);
    return x.readUIntBE(0, 3).toString(16);
  })(),
); // "abcdef"

console.log("\n=== Byte swapping ===");

const sw = Buffer.from([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
sw.swap32();
log("swap32", Array.from(sw)); // [ 4, 3, 2, 1, 8, 7, 6, 5 ]
const sw2 = Buffer.from([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
sw2.swap64();
log("swap64", Array.from(sw2)); // [ 8, 7, 6, 5, 4, 3, 2, 1 ]
// swap* throws if the length is not a multiple of the width:
let threw = false;
try {
  Buffer.from([1, 2, 3]).swap16();
} catch {
  threw = true;
}
log("swap16 throws on odd length", threw); // true

console.log("\n=== Binary protocol round-trip ===");

// A tiny message format:
//   u16 LE  message type
//   u8      flags
//   u32 BE  payload length
//   f64 BE  timestamp
function encode(type: number, flags: number, payloadLen: number, ts: number): Buffer {
  const msg = Buffer.alloc(15);
  let o = msg.writeUInt16LE(type, 0); // +2
  o = msg.writeUInt8(flags, o); // +1
  o = msg.writeUInt32BE(payloadLen, o); // +4
  o = msg.writeDoubleBE(ts, o); // +8  -> 15
  return msg;
}

function decode(msg: Buffer): { type: number; flags: number; payloadLen: number; ts: number } {
  let o = 0;
  const type = msg.readUInt16LE(o);
  o += 2;
  const flags = msg.readUInt8(o);
  o += 1;
  const payloadLen = msg.readUInt32BE(o);
  o += 4;
  const ts = msg.readDoubleBE(o);
  o += 8;
  return { type, flags, payloadLen, ts };
}

const wire = encode(0xbeef, 0x07, 42, 1.733e9);
log("encoded wire bytes", Array.from(wire)); // 15 bytes
log("decoded", decode(wire)); // { type: 48879, flags: 7, payloadLen: 42, ts: 1733000000000 }

console.log("\nAll binary I/O examples executed successfully.");
