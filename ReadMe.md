# Buffer & Rocket Native Bindings

High-performance native Node.js `node:buffer` implementation backed by Rust `bytes` crate and native **Rocket Web Framework** NAPI bindings with code generation support.

## Features

- **`Buffer` Class:** Full `Uint8Array` subclass providing Node.js `Buffer` static and instance methods.
- **Fast Encodings:** Support for `utf8`, `ascii`, `latin1`/`binary`, `hex`, `base64`, `base64url`, and `utf16le`/`ucs2`.
- **Numeric & BigInt I/O:** Complete big-endian and little-endian 8-bit, 16-bit, 32-bit, 64-bit, and variable byte length (1–6 bytes) integer/float/double read and write operations.
- **`Blob` & `File` Classes:** Web-compatible `Blob` and `File` implementations with `arrayBuffer()`, `bytes()`, `text()`, and `slice()`.
- **Rocket Web Framework Integration:** Native NAPI bindings for the Rocket web framework (`RocketServer`), supporting server configuration, route mounting with JavaScript callbacks, request/response models (`JsRequest`, `JsResponse`), and dispatch testing.
- **Rocket Code Generation:** Support for parsing route/catcher attributes (`#[get]`, `#[catch]`), generating macros (`routes!`, `catchers!`, `uri!`, `#[launch]`), and compile-time/runtime debug logging (`ROCKET_CODEGEN_DEBUG`).
- **Utility Functions:** Native `atob`, `btoa`, `isAscii`, `isUtf8`, `transcode`, and module constants (`constants`, `kMaxLength`, `kStringMaxLength`, `INSPECT_MAX_BYTES`, `SlowBuffer`).

## Usage

### Buffer & Utility Usage

```ts
import { Buffer, Blob, File, atob, btoa, isAscii, isUtf8, transcode } from "@alloy-ts/buffer";

// Create zero-filled Buffer of length 10
const buf1 = Buffer.alloc(10);

// Create Buffer filled with byte value 1
const buf2 = Buffer.alloc(10, 1);

// Create uninitialized Buffer
const buf3 = Buffer.allocUnsafe(10);

// Create Buffer from byte array
const buf4 = Buffer.from([1, 2, 3]);

// Create Buffer from UTF-8 string
const buf5 = Buffer.from("tést");
console.log(buf5.toString("hex")); // "74c3a97374"

// Create Buffer from Latin-1 string
const buf6 = Buffer.from("tést", "latin1");

// Web-compatible Blob and File
const blob = new Blob(["hello ", "world"], { type: "text/plain" });
console.log(blob.size); // 11
console.log(blob.text()); // "hello world"

// Base64 encoding / decoding
const b64 = btoa("hello"); // "aGVsbG8="
const str = atob(b64); // "hello"

// Character encoding transcoding
const transcoded = transcode(Buffer.from("€"), "utf8", "ascii");
console.log(transcoded.toString("ascii")); // "?"
```

### Rocket Web Framework & Code Generation Usage

```ts
import {
  RocketServer,
  createJsonResponse,
  parseRouteAttribute,
  generateRouteCode,
  generateRoutesMacro,
  generateUriMacro,
} from "@alloy-ts/buffer";

// Initialize Rocket server
const app = RocketServer.build();

// Mount a route with a JavaScript handler
app.mountRoute(
  "/user",
  { method: "GET", path: "/<id>" },
  (_err, req) => {
    return createJsonResponse(
      200,
      JSON.stringify({ userId: req.params.id, status: "active" })
    );
  }
);

// Dispatch requests directly (for testing or server execution)
const response = await app.dispatch("GET", "/user/42", null);
console.log(response.status); // 200
console.log(response.body?.toString("utf8")); // '{"userId":"42","status":"active"}'

// Rocket Code Generation
const meta = parseRouteAttribute("GET", '"/<foo>/bar/<baz..>?<msg>", rank = 1, format = "json"');
console.log(meta.pathSegments); // [{ name: 'foo', isTrailing: false }, { name: 'baz', isTrailing: true }]

const routeCode = generateRouteCode("GET", "/person/<name>", "getPerson", 1, "json");
console.log(routeCode.rustCode); // '#[get("/person/<name>", rank = 1, format = "json")]\nfn getPerson() ...'

const uriCode = generateUriMacro("person", { name: "Mike", age: "28" });
console.log(uriCode.tsCode); // 'export const rocket_uri_macro_person = "/person?name=Mike&age=28";'
```

## Development

- Configure local hooks:

```bash
npm run prepare
```

- Install dependencies:

```bash
npm install
```

- Run unit tests (Rust and TypeScript):

```bash
npm test
```

- Build native addon & bundle library:

```bash
npm run build
```

- Code check (formatting & linting):

```bash
npm run check
```

- Format code:

```bash
npm run fmt
```

- Lint code:

```bash
npm run lint
```

## API Documentation

For the complete API compatibility matrix and review against Node.js `node:buffer` specifications, see [docs/api.md](docs/api.md).
