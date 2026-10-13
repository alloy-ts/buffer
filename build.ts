import { NapiCli } from "@napi-rs/cli";

async function run() {
  const args = process.argv.slice(2);
  const isRelease = args.includes("--release") || args.includes("-r");
  const useNapiCross = args.includes("--use-napi-cross");
  const crossCompile = args.includes("--cross-compile") || args.includes("-x");
  const useCross = args.includes("--use-cross");

  const targetIdx = args.findIndex((a) => a === "--target" || a === "-t");
  const target = targetIdx !== -1 && args[targetIdx + 1] ? args[targetIdx + 1] : undefined;

  const cli = new NapiCli();
  await cli.build({
    platform: true,
    esm: true,
    outputDir: "./dist",
    release: isRelease,
    target,
    useNapiCross,
    crossCompile,
    useCross,
  });

  const fs = await import("node:fs");
  if (fs.existsSync("./dist/my-addon.linux-x64-gnu.node")) {
    fs.copyFileSync("./dist/my-addon.linux-x64-gnu.node", "./my-addon.linux-x64-gnu.node");
  }
  if (fs.existsSync("./dist/index.js")) {
    fs.copyFileSync("./dist/index.js", "./index.js");
  }
  if (fs.existsSync("./dist/index.d.ts")) {
    fs.copyFileSync("./dist/index.d.ts", "./index.d.ts");
  }
}

void run().catch((err) => {
  console.error(err);
  process.exit(1);
});
