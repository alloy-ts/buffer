import { expect, test } from "vite-plus/test";
import {
  parseRouteAttribute,
  parseCatcherAttribute,
  generateRouteCode,
  generateRoutesMacro,
  generateCatchersMacro,
  generateUriMacro,
  generateLaunchCode,
  emitCodegenDebug,
} from "../index.js";

test("parseRouteAttribute - dynamic parameters, rank, format, data", () => {
  const meta = parseRouteAttribute(
    "GET",
    '"/<foo>/bar/<baz..>?<msg>", data = "<form>", rank = 2, format = "json"'
  );

  expect(meta.method).toBe("GET");
  expect(meta.uri).toBe("/<foo>/bar/<baz..>?<msg>");
  expect(meta.pathSegments.length).toBe(2);
  expect(meta.pathSegments[0]).toEqual({ name: "foo", isTrailing: false });
  expect(meta.pathSegments[1]).toEqual({ name: "baz", isTrailing: true });

  expect(meta.querySegments.length).toBe(1);
  expect(meta.querySegments[0]).toEqual({ name: "msg", isTrailing: false });

  expect(meta.rank).toBe(2);
  expect(meta.format).toBe("json");
  expect(meta.dataParam).toBe("form");
});

test("parseCatcherAttribute - status code and default", () => {
  const notFound = parseCatcherAttribute("404");
  expect(notFound.statusCode).toBe(404);
  expect(notFound.isDefault).toBe(false);

  const defaultCatcher = parseCatcherAttribute("default");
  expect(defaultCatcher.statusCode).toBeUndefined();
  expect(defaultCatcher.isDefault).toBe(true);
});

test("generateRouteCode - Rust and TS route definitions", () => {
  const res = generateRouteCode("GET", "/person/<name>", "getPerson", 1, "json");
  expect(res.rustCode).toContain('#[get("/person/<name>", rank = 1, format = "json")]');
  expect(res.rustCode).toContain("fn getPerson()");
  expect(res.tsCode).toContain('method: "GET"');
  expect(res.tsCode).toContain('path: "/person/<name>"');
});

test("generateRoutesMacro and generateCatchersMacro", () => {
  const routesRes = generateRoutesMacro(["index", "person::hello"]);
  expect(routesRes.rustCode).toBe("routes![index, person::hello]");
  expect(routesRes.tsCode).toBe("export const routes = [index, person::hello];");

  const catchersRes = generateCatchersMacro(["not_found", "unauthorized", "default_catcher"]);
  expect(catchersRes.rustCode).toBe("catchers![not_found, unauthorized, default_catcher]");
  expect(catchersRes.tsCode).toBe("export const catchers = [not_found, unauthorized, default_catcher];");
});

test("generateUriMacro - type-safe route URI generation", () => {
  const uriRes = generateUriMacro("person", { name: "Mike", age: "28" });
  expect(uriRes.rustCode).toContain('uri!(person(name = "Mike", age = "28"))');
  expect(uriRes.tsCode).toContain('rocket_uri_macro_person = "/person?name=Mike&age=28"');
});

test("generateLaunchCode - #[launch] main function", () => {
  const launchRes = generateLaunchCode("rocket");
  expect(launchRes.rustCode).toContain("#[launch]");
  expect(launchRes.rustCode).toContain("fn rocket() -> _");
  expect(launchRes.tsCode).toContain("export function launchApp()");
});

test("emitCodegenDebug - ROCKET_CODEGEN_DEBUG environment variable", () => {
  process.env.ROCKET_CODEGEN_DEBUG = "1";
  const debugOutput = emitCodegenDebug("test debug log");
  expect(debugOutput).toBe("[ROCKET_CODEGEN_DEBUG] test debug log");
  delete process.env.ROCKET_CODEGEN_DEBUG;
});
