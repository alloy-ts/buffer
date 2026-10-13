import console from "node:console";

export {
  Buffer,
  Blob,
  File,
  Bytes,
  Server,
  add,
  atob,
  btoa,
  constants,
  inspectMaxBytes,
  isAscii,
  isLatin1,
  isUtf8,
  kMaxLength,
  kStringMaxLength,
  resolveObjectUrl,
  transcode,
} from "../index.js";

export const main = () => {
  return "Hello, world!";
};

console.log(main());
