import { NapiCli } from "@napi-rs/cli";
import fs from "node:fs";
import path from "node:path";

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

  if (fs.existsSync("./dist")) {
    for (const file of fs.readdirSync("./dist")) {
      const srcFile = path.join("./dist", file);
      const destFile = path.join(".", file);
      if (fs.existsSync(destFile)) {
        try {
          fs.unlinkSync(destFile);
        } catch {
          // ignore error if unlink fails
        }
      }
      fs.copyFileSync(srcFile, destFile);
    }
  }
}

void run().catch((err) => {
  console.error(err);
  process.exit(1);
});
