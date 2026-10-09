import {
  Buffer as NodeBuffer,
  Blob as NodeBlob,
  File as NodeFile,
  atob as nodeAtob,
  btoa as nodeBtoa,
  isAscii as nodeIsAscii,
  isUtf8 as nodeIsUtf8,
  resolveObjectURL as nodeResolveObjectURL,
  transcode as nodeTranscode,
  constants as nodeConstants,
  INSPECT_MAX_BYTES as nodeInspectMaxBytes,
  kMaxLength as nodeKMaxLength,
  kStringMaxLength as nodeKStringMaxLength,
} from "node:buffer";

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

export interface BlobOptions {
  endings?: "transparent" | "native";
  type?: string;
}

export interface FileOptions extends BlobOptions {
  lastModified?: number;
}

export type WithImplicitCoercion<T> =
  | T
  | { valueOf(): T }
  | (T extends string ? { [Symbol.toPrimitive](hint: "string"): T } : never);

export type ImplicitArrayBuffer<T extends WithImplicitCoercion<ArrayBufferLike>> = T extends {
  valueOf(): infer V extends ArrayBufferLike;
}
  ? V
  : T;

export const Buffer = NodeBuffer;
export type Buffer = NodeBuffer;

export const Blob = NodeBlob;
export type Blob = NodeBlob;

export const File = NodeFile;
export type File = NodeFile;

export const atob = nodeAtob;
export const btoa = nodeBtoa;
export const isAscii = nodeIsAscii;
export const isUtf8 = nodeIsUtf8;
export const resolveObjectURL = nodeResolveObjectURL;
export const transcode = nodeTranscode;

export const constants = nodeConstants;
export const INSPECT_MAX_BYTES = nodeInspectMaxBytes;
export const kMaxLength = nodeKMaxLength;
export const kStringMaxLength = nodeKStringMaxLength;
// SlowBuffer is deprecated in Node.js and aliased to Buffer constructor in runtime
export const SlowBuffer = NodeBuffer;

export default {
  Buffer,
  Blob,
  File,
  atob,
  btoa,
  isAscii,
  isUtf8,
  resolveObjectURL,
  transcode,
  constants,
  INSPECT_MAX_BYTES,
  kMaxLength,
  kStringMaxLength,
  SlowBuffer,
};
