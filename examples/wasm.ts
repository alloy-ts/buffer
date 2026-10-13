/**
 * examples/wasm.ts
 *
 * Demonstrates using `@alloy-ts/buffer` with WebAssembly (WASM / WASI) targets.
 *
 *   node --import @oxc-node/core/register examples/wasm.ts
 */
import {
  Buffer,
  Blob,
  File,
  atob,
  btoa,
  isAscii,
  isUtf8,
  transcode,
  __napiBindingTarget,
} from "../dist/index.js";

const log = (label: string, value: unknown): void => {
  console.log(`\x1b[36m${label}\x1b[0m`, value);
};

console.log("=== WASM / Native Binding Information ===");
log("Loaded NAPI binding target", __napiBindingTarget);

console.log("\n=== Buffer I/O via WASM/NAPI ===");
const buf = Buffer.from("Hello from WASM / NAPI!");
log("Buffer content", buf.toString());
log("Buffer byte length", buf.byteLength);
log("Buffer hex", buf.toHex());

console.log("\n=== Web Standards in WASM ===");
const blob = new Blob(["WASM ", "Blob ", "content"], { type: "text/plain" });
log("Blob size", blob.size);
log("Blob text", blob.text());

const file = new File(["WASM file data"], "wasm_file.txt", { type: "text/plain" });
log("File name", file.name);
log("File text", file.text());

console.log("\n=== Encoding Utilities in WASM ===");
const b64 = btoa("WASM Buffer");
log("btoa('WASM Buffer')", b64);
log("atob(b64)", atob(b64));

const utf8Buf = Buffer.from("WASM");
log("isAscii", isAscii(new Uint8Array([87, 65, 83, 77])));
log("isUtf8", isUtf8(new Uint8Array([87, 65, 83, 77])));

console.log("\nAll WebAssembly example operations completed successfully.");
