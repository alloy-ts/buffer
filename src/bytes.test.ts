import { expect, test } from "vite-plus/test";
import { Bytes } from "./main.ts";

// `Bytes` is a native (napi-rs) export; importing it from the public entry
// exercises the same binding consumers will use.

// Helper: create a byte source accepted by `Bytes.fromBuffer`. The native
// binding accepts any napi `Buffer` or `Uint8Array` at runtime; we type the
// return as the expected parameter type so the call sites stay strict.
function nodeBuf(bytes: number[]): Parameters<typeof Bytes.fromBuffer>[0] {
  return new Uint8Array(bytes) as unknown as Parameters<typeof Bytes.fromBuffer>[0];
}

test("Bytes constructor is empty and does not allocate contents", () => {
  const b = new Bytes();
  expect(b.len()).toBe(0);
  expect(b.isEmpty()).toBe(true);
  expect(Array.from(b.buf())).toEqual([]);
});

test("Bytes.fromBuffer copies the contents of the source", () => {
  const b = Bytes.fromBuffer(nodeBuf([1, 2, 3, 4]));
  expect(b.isEmpty()).toBe(false);
  expect(b.len()).toBe(4);
  expect(Array.from(b.buf())).toEqual([1, 2, 3, 4]);
});

test("Bytes.asSlice and Bytes.buf return independent copies", () => {
  const b = Bytes.fromBuffer(nodeBuf([10, 20, 30]));
  const a = b.asSlice();
  const c = b.buf();
  expect(Array.from(a)).toEqual([10, 20, 30]);
  expect(Array.from(c)).toEqual([10, 20, 30]);
  // They are copies, not the same object.
  expect(a).not.toBe(c);
});

test("Bytes.slice returns the half-open [start, end) range", () => {
  const b = Bytes.fromBuffer(nodeBuf([0, 1, 2, 3, 4, 5]));
  const s = b.slice(1, 4);
  expect(s.len()).toBe(3);
  expect(Array.from(s.buf())).toEqual([1, 2, 3]);

  // Omitting `end` defaults to the full length.
  const s2 = b.slice(2);
  expect(s2.len()).toBe(4);
  expect(Array.from(s2.buf())).toEqual([2, 3, 4, 5]);
});

test("Bytes.slice throws on out-of-range indices", () => {
  const b = Bytes.fromBuffer(nodeBuf([0, 1, 2, 3]));
  expect(() => b.slice(2, 100)).toThrow();
  expect(() => b.slice(5, 6)).toThrow();
});

test("Bytes.splitOff keeps [0, at) and returns [at, len)", () => {
  const b = Bytes.fromBuffer(nodeBuf([0, 1, 2, 3, 4]));
  const tail = b.splitOff(3);
  expect(Array.from(b.buf())).toEqual([0, 1, 2]);
  expect(Array.from(tail.buf())).toEqual([3, 4]);
  expect(() => b.splitOff(100)).toThrow();
});

test("Bytes.splitTo keeps [at, len) and returns [0, at)", () => {
  const b = Bytes.fromBuffer(nodeBuf([0, 1, 2, 3, 4]));
  const head = b.splitTo(2);
  expect(Array.from(head.buf())).toEqual([0, 1]);
  expect(Array.from(b.buf())).toEqual([2, 3, 4]);
  expect(() => b.splitTo(100)).toThrow();
});

test("Bytes.truncate keeps the first n bytes", () => {
  const b = Bytes.fromBuffer(nodeBuf([1, 2, 3, 4, 5]));
  b.truncate(2);
  expect(Array.from(b.buf())).toEqual([1, 2]);
  // Truncating beyond the length is a no-op.
  b.truncate(99);
  expect(Array.from(b.buf())).toEqual([1, 2]);
});

test("Bytes.clear empties the buffer", () => {
  const b = Bytes.fromBuffer(nodeBuf([7, 8, 9]));
  b.clear();
  expect(b.len()).toBe(0);
  expect(b.isEmpty()).toBe(true);
});

test("Bytes.tryIntoMut returns a view when uniquely owned", () => {
  // A freshly created, unshared `Bytes` is the unique owner.
  const b = Bytes.fromBuffer(nodeBuf([7, 8, 9]));
  const m = b.tryIntoMut();
  expect(m).not.toBeNull();
  expect(m!.len()).toBe(3);

  // Once the handle has been cloned (e.g. via slice), it is no longer unique.
  const shared = Bytes.fromBuffer(nodeBuf([1, 2, 3]));
  shared.slice(0, 1);
  expect(shared.tryIntoMut()).toBeNull();
});
