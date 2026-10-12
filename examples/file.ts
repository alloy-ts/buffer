import { File } from "../src/main.ts";

async function run() {
  console.log("=== File Examples ===");

  // 1. Create a File instance
  const file = new File(["Hello, File API!"], "hello.txt", {
    type: "text/plain",
    lastModified: Date.now(),
  });

  console.log("File name:", file.name);
  console.log("File size in bytes:", file.size);
  console.log("File type:", file.type);
  console.log("File lastModified:", new Date(file.lastModified).toISOString());

  // 2. Read File content asynchronously
  const text = await file.text();
  console.log("File content (text):", text);

  const bytes = await file.bytes();
  console.log("File content (bytes):", bytes);

  const buffer = await file.arrayBuffer();
  console.log("File content (ArrayBuffer length):", buffer.byteLength);

  // 3. Slice a File
  const sliced = file.slice(0, 5);
  console.log("Sliced File text:", await sliced.text());
}

void run();
