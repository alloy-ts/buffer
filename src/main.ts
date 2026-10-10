import * as native from "../dist/index.js";

export const Buffer = native.Buffer;
export const Blob = native.Blob;
export const File = native.File;

export const atob = native.atob;
export const btoa = native.btoa;
export const isAscii = native.isAscii;
export const isUtf8 = native.isUtf8;
export const resolveObjectURL = native.resolveObjectURL;
export const transcode = native.transcode;

export const constants = (native as any).constants ?? {
  MAX_LENGTH: 2147483647,
  MAX_STRING_LENGTH: 536870888,
};

export const INSPECT_MAX_BYTES = (native as any).INSPECT_MAX_BYTES ?? 50;
export const kMaxLength = (native as any).kMaxLength ?? 2147483647;
export const kStringMaxLength = (native as any).kStringMaxLength ?? 536870888;
export const poolSize = (native as any).poolSize ?? 65536;

export const SlowBuffer = function (size: number) {
  return Buffer.allocUnsafeSlow(size);
};
SlowBuffer.prototype = Buffer.prototype;

export type {
  BlobOptions,
  FileOptions,
  BufferJson,
} from "../dist/index.d.ts";
