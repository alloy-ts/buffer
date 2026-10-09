import * as mod from "node:buffer";

export const Blob = mod.Blob;
export const Buffer = mod.Buffer;
export const File = mod.File;
export const INSPECT_MAX_BYTES = mod.INSPECT_MAX_BYTES;
export const SlowBuffer = (mod as { SlowBuffer?: typeof mod.Buffer }).SlowBuffer ?? mod.Buffer;
export const atob = mod.atob;
export const btoa = mod.btoa;
export const constants = mod.constants;
export const isAscii = mod.isAscii;
export const isUtf8 = mod.isUtf8;
export const kMaxLength = mod.kMaxLength;
export const kStringMaxLength = mod.kStringMaxLength;
export const resolveObjectURL = mod.resolveObjectURL;
export const transcode = mod.transcode;

export type Blob = InstanceType<typeof mod.Blob>;
export type Buffer = mod.Buffer;
export type File = InstanceType<typeof mod.File>;

export type BufferConstructor = typeof mod.Buffer;

export interface BlobOptions {
  endings?: "transparent" | "native";
  type?: string;
}

export interface FileOptions {
  endings?: "transparent" | "native";
  lastModified?: number;
  type?: string;
}

export type BufferEncoding =
  | "ascii"
  | "utf8"
  | "utf-8"
  | "utf16le"
  | "utf-16le"
  | "ucs2"
  | "ucs-2"
  | "base64"
  | "base64url"
  | "latin1"
  | "binary"
  | "hex";

export type TranscodeEncoding =
  | "ascii"
  | "utf8"
  | "utf-8"
  | "utf16le"
  | "utf-16le"
  | "ucs2"
  | "ucs-2"
  | "latin1"
  | "binary";

export type WithImplicitCoercion<T> =
  | T
  | { valueOf(): T }
  | (T extends string ? { [Symbol.toPrimitive](hint: "string"): T } : never);

export type ImplicitArrayBuffer<T extends WithImplicitCoercion<ArrayBufferLike>> = T extends {
  valueOf(): infer V extends ArrayBufferLike;
}
  ? V
  : T;
