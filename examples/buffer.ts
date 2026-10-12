import { Buffer } from "../src/main.ts";

// 1. Creating Buffers
const bufAlloc = Buffer.alloc(10, 1);
console.log("Buffer.alloc(10, 1):", bufAlloc);

const bufString = Buffer.from("Hello World!", "utf8");
console.log("Buffer.from string:", bufString, "->", bufString.toString("utf8"));

const bufHex = Buffer.from("48656c6c6f", "hex");
console.log("Buffer.from hex:", bufHex.toString("utf8"));

// 2. Read / Write numbers
const numBuf = Buffer.alloc(8);
numBuf.writeUInt32BE(0x12345678, 0);
numBuf.writeFloatLE(123.456, 4);
console.log("Read UInt32BE:", numBuf.readUInt32BE(0).toString(16));
console.log("Read FloatLE:", numBuf.readFloatLE(4));

// 3. Buffer operations
const slice = bufString.subarray(0, 5);
console.log("Subarray 0..5:", slice.toString());

const concat = Buffer.concat([bufString, Buffer.from(" Extra")]);
console.log("Concat:", concat.toString());
