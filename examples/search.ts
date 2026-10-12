/**
 * examples/search.ts
 *
 * Demonstrates searching/comparison on `@alloy-ts/buffer` `Buffer`s:
 * `indexOf` / `lastIndexOf` / `includes` (with encodings), `equals`,
 * ranged `compare`, `Buffer.from(object)` coercion, `toJSON`, `concat`
 * truncation/padding, and `slice` vs `subarray` memory sharing.
 *
 * Run with oxc-node:
 *
 *   node --import @oxc-node/core/register examples/search.ts
 */
import { Buffer } from "../index.js";

const log = (label: string, value: unknown): void => {
  console.log(`\x1b[36m${label}\x1b[0m`, value);
};

console.log("=== indexOf / lastIndexOf / includes ===");

const hay = Buffer.from("the quick brown fox jumps over the lazy dog");
log('indexOf("fox")', hay.indexOf("fox")); // 16
log('lastIndexOf("the")', hay.lastIndexOf("the")); // 35
log('includes("lazy")', hay.includes("lazy")); // true
// Search from a byte offset
log('indexOf("o") from 20', hay.indexOf("o", 20)); // 26
// Search by byte value
log("indexOf(0x20 /* space */)", hay.indexOf(0x20)); // 3
// Search by Buffer needle
log("indexOf(Buffer)", hay.indexOf(Buffer.from("brown"))); // 10

console.log("\n=== equals & ranged compare ===");

const x = Buffer.from("abcdef");
const y = Buffer.from("abcdef");
log("equals (true)", x.equals(y)); // true
log("equals (false)", x.equals(Buffer.from("abcdeg"))); // false

// compare(this[sourceStart:sourceEnd], target[targetStart:targetEnd])
// here: x[2:5]="cde" vs y[0:3]="abc" -> "cde" > "abc" -> 1
log("ranged compare x.cmp(y, 0,3, 2,5)", x.compare(y, 0, 3, 2, 5)); // 1
log("ranged compare equal", x.compare(y, 1, 4, 1, 4)); // 0

console.log("\n=== Buffer.from(object) coercion ===");

// Objects with valueOf() are coerced via it.
const withValueOf = { valueOf() { return "ABC"; } };
log("from valueOf object", Buffer.from(withValueOf).toString()); // "ABC"

// Objects with Symbol.toPrimitive are coerced via it.
const withToPrimitive = { [Symbol.toPrimitive]() { return "XYZ"; } };
log("from Symbol.toPrimitive object", Buffer.from(withToPrimitive).toString()); // "XYZ"

console.log("\n=== toJSON ===");

log("toJSON", Buffer.from("abc").toJSON()); // { type: "Buffer", data: [ 97, 98, 99 ] }
log("JSON.stringify", JSON.stringify(Buffer.from("abc"))); // {"type":"Buffer","data":[97,98,99]}

console.log("\n=== concat truncation / padding ===");

const a = Buffer.from("aaa");
const c = Buffer.from("ccc");
// Default: totalLength = sum of parts.
log("concat default length", Buffer.concat([a, c]).length); // 6
// Truncation when totalLength < sum.
log("concat truncated to 4", Buffer.concat([a, c], 4).toString()); // "aaac"
// Padding with zeros when totalLength > sum.
const padded = Buffer.concat([a, c], 8);
log("concat padded to 8 (zeros)", Array.from(padded)); // [ a a a c c c 0 0 ]

console.log("\n=== slice vs subarray (memory sharing) ===");

const src = Buffer.from([1, 2, 3, 4, 5]);
const sub = src.subarray(1, 4);
sub[0] = 99; // mutating the view mutates the underlying memory
log("src after subarray mutate", Array.from(src)); // [ 1, 99, 3, 4, 5 ]

const slc = src.slice(1, 4);
slc[1] = 88; // slice is an alias of subarray here, so it also shares memory
log("src after slice mutate", Array.from(src)); // [ 1, 99, 88, 4, 5 ]

console.log("\nAll search/compare examples executed successfully.");
