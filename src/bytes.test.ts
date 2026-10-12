import { expect, test } from "vite-plus/test";
import { Buffer, Bytes } from "./main.ts";

test("Bytes constructor and fromBuffer", () => {
  const empty = new Bytes();
  expect(empty.len()).toBe(0);
  expect(empty.isEmpty()).toBe(true);

  const buf = Buffer.from([10, 20, 30, 40, 50]);
  const bytes = Bytes.fromBuffer(buf);
  expect(bytes.len()).toBe(5);
  expect(bytes.isEmpty()).toBe(false);
  expect(Array.from(bytes.asSlice())).toEqual([10, 20, 30, 40, 50]);
  expect(Array.from(bytes.buf())).toEqual([10, 20, 30, 40, 50]);
});

test("Bytes isUnique and tryIntoMut", () => {
  const buf = Buffer.from([1, 2, 3]);
  const bytes = Bytes.fromBuffer(buf);
  expect(bytes.isUnique()).toBe(true);

  const mutBytes = bytes.tryIntoMut();
  expect(mutBytes).not.toBeNull();
  expect(Array.from(mutBytes!.asSlice())).toEqual([1, 2, 3]);
});

test("Bytes slice", () => {
  const buf = Buffer.from([10, 20, 30, 40, 50]);
  const bytes = Bytes.fromBuffer(buf);

  const sliced1 = bytes.slice(1, 4);
  expect(sliced1.len()).toBe(3);
  expect(Array.from(sliced1.asSlice())).toEqual([20, 30, 40]);

  const sliced2 = bytes.slice(2);
  expect(sliced2.len()).toBe(3);
  expect(Array.from(sliced2.asSlice())).toEqual([30, 40, 50]);

  expect(() => bytes.slice(4, 2)).toThrow();
  expect(() => bytes.slice(0, 10)).toThrow();
});

test("Bytes splitOff", () => {
  const buf = Buffer.from([10, 20, 30, 40, 50]);
  const bytes = Bytes.fromBuffer(buf);

  const right = bytes.splitOff(2);
  expect(bytes.len()).toBe(2);
  expect(Array.from(bytes.asSlice())).toEqual([10, 20]);
  expect(right.len()).toBe(3);
  expect(Array.from(right.asSlice())).toEqual([30, 40, 50]);

  expect(() => right.splitOff(10)).toThrow();
});

test("Bytes splitTo", () => {
  const buf = Buffer.from([10, 20, 30, 40, 50]);
  const bytes = Bytes.fromBuffer(buf);

  const left = bytes.splitTo(2);
  expect(left.len()).toBe(2);
  expect(Array.from(left.asSlice())).toEqual([10, 20]);
  expect(bytes.len()).toBe(3);
  expect(Array.from(bytes.asSlice())).toEqual([30, 40, 50]);

  expect(() => bytes.splitTo(10)).toThrow();
});

test("Bytes truncate and clear", () => {
  const buf = Buffer.from([10, 20, 30, 40, 50]);
  const bytes = Bytes.fromBuffer(buf);

  bytes.truncate(3);
  expect(bytes.len()).toBe(3);
  expect(Array.from(bytes.asSlice())).toEqual([10, 20, 30]);

  bytes.clear();
  expect(bytes.len()).toBe(0);
  expect(bytes.isEmpty()).toBe(true);
});
