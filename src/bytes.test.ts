import { expect, test } from "vite-plus/test";
import { Bytes } from "../index.js";
import { Buffer } from "./main.ts";

test("Bytes constructor, fromBuffer, len, isEmpty, and isUnique", () => {
  const emptyBytes = new Bytes();
  expect(emptyBytes.len()).toBe(0);
  expect(emptyBytes.isEmpty()).toBe(true);

  const buf = Buffer.from("hello world");
  const bytes = Bytes.fromBuffer(buf);
  expect(bytes.len()).toBe(11);
  expect(bytes.isEmpty()).toBe(false);
  expect(typeof bytes.isUnique()).toBe("boolean");
});

test("Bytes.slice, splitOff, and splitTo", () => {
  const buf = Buffer.from("hello world");
  const bytes = Bytes.fromBuffer(buf);

  const sliced = bytes.slice(0, 5);
  expect(sliced.len()).toBe(5);
  expect(sliced.asSlice().toString()).toBe("hello");

  let mutBytes1 = Bytes.fromBuffer(Buffer.from("hello world"));
  const splitOffRes = mutBytes1.splitOff(5);
  expect(mutBytes1.len()).toBe(5);
  expect(mutBytes1.asSlice().toString()).toBe("hello");
  expect(splitOffRes.len()).toBe(6);
  expect(splitOffRes.asSlice().toString()).toBe(" world");

  let mutBytes2 = Bytes.fromBuffer(Buffer.from("hello world"));
  const splitToRes = mutBytes2.splitTo(5);
  expect(mutBytes2.len()).toBe(6);
  expect(mutBytes2.asSlice().toString()).toBe(" world");
  expect(splitToRes.len()).toBe(5);
  expect(splitToRes.asSlice().toString()).toBe("hello");
});

test("Bytes.truncate, clear, tryIntoMut, asSlice, and buf", () => {
  let bytes = Bytes.fromBuffer(Buffer.from("hello world"));

  bytes.truncate(5);
  expect(bytes.len()).toBe(5);
  expect(bytes.asSlice().toString()).toBe("hello");

  const tryMut = bytes.tryIntoMut();
  expect(tryMut).not.toBeNull();
  if (tryMut) {
    expect(tryMut.len()).toBe(5);
  }

  const bufCopy = bytes.buf();
  expect(bufCopy.toString()).toBe("hello");

  bytes.clear();
  expect(bytes.len()).toBe(0);
  expect(bytes.isEmpty()).toBe(true);
});
