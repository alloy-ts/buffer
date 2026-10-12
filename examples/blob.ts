import { Blob, File, resolveObjectURL } from "../src/main.ts";

async function run() {
  console.log("=== Blob & File Examples ===");

  // 1. Blob creation and async text/bytes
  const blob = new Blob(["hello ", "world!"], { type: "text/plain" });
  console.log("Blob size:", blob.size);
  console.log("Blob type:", blob.type);
  console.log("Blob text():", await blob.text());

  const bytes = await blob.bytes();
  console.log("Blob bytes():", bytes);

  const ab = await blob.arrayBuffer();
  console.log("Blob arrayBuffer byteLength:", ab.byteLength);

  // 2. Blob stream
  const stream = blob.stream();
  const reader = stream.getReader();
  const chunk = await reader.read();
  console.log("Blob stream chunk:", chunk.value);

  // 3. Blob slicing
  const sliced = blob.slice(0, 5);
  console.log("Sliced Blob text():", await sliced.text());

  // 4. File class
  const file = new File(["file content"], "example.txt", { type: "text/plain" });
  console.log("File name:", file.name);
  console.log("File size:", file.size);
  console.log("File lastModified:", file.lastModified);
  console.log("File text():", await file.text());

  // 5. Blob URL resolution
  if (typeof URL !== "undefined" && typeof URL.createObjectURL === "function") {
    const url = URL.createObjectURL(blob as any);
    console.log("Created object URL:", url);
    const resolved = resolveObjectURL(url);
    console.log("Resolved Object URL text():", await resolved?.text());
    URL.revokeObjectURL(url);
  }
}

void run();
