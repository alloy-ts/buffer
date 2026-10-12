# Native Node.js Buffer API Bindings

NAPI-RS native Rust bindings providing a high-performance `node:buffer` compatible API.

## Features

- **`Buffer`**: Full implementation of Node.js `Buffer` class methods, including reading/writing binary primitives (integers, floats, doubles, BigInts), conversions (string, JSON, hex, base64, latin1, utf16le), copying, slicing, swapping, and array operations.
- **`Blob` & `File`**: Standard Web/Node.js `Blob` and `File` class implementations.
- **Utility Functions**: Includes `atob`, `btoa`, `isAscii`, `isLatin1`, `isUtf8`, `transcode`, `resolveObjectURL`, and `constants`.

For a full list of supported APIs and implementation compatibility comparison, see [docs/api.md](docs/api.md).

## Development

- Configure local hooks:

```bash
npm run prepare
```

- Install dependencies:

```bash
vp install
```

- Run the unit tests:

```bash
vp test
```

- Run locally:

```bash
npm run dev
```

- Build the library:

```bash
npm run build
```

- Code formatting:

```bash
npm run fmt
```

- Linting:

```bash
npm run lint
```

- Code check:

```bash
npm run check
```
