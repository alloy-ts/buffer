import { expect, test } from "vite-plus/test";
import {
  RouteCodegen,
  CatcherCodegen,
  FormCodegen,
  FormFieldEnumCodegen,
  ResponderCodegen,
  UriCodegen,
  RocketApp,
  routes,
  catchers,
  generateRoute,
  generateCatcher,
  generateUri,
  setCodegenDebug,
  isCodegenDebugEnabled,
  HttpMethod,
} from "./main.ts";

test("RouteCodegen and routes helper", () => {
  const route1 = new RouteCodegen("GET", "/hello/<name>?<age>", "hello", {
    rank: 1,
    format: "json",
  });
  const meta1 = route1.generate();

  expect(meta1.method).toBe("GET");
  expect(meta1.path).toBe("/hello/<name>?<age>");
  expect(meta1.name).toBe("hello");
  expect(meta1.rank).toBe(1);
  expect(meta1.format).toBe("json");
  expect(meta1.pathParams).toEqual(["name"]);
  expect(meta1.queryParams).toEqual(["age"]);

  const route2 = new RouteCodegen("POST", "/user", "create_user", {
    data: "<user_form>",
  });
  const metaList = routes([route1, route2]);

  expect(metaList.length).toBe(2);
  expect(metaList[1]!.method).toBe("POST");
  expect(metaList[1]!.data).toBe("<user_form>");

  const routeGen = generateRoute(HttpMethod.Get, "/index", "index");
  expect(routeGen.name).toBe("index");
  expect(routeGen.method).toBe("GET");
});

test("RouteCodegen emitDebug", () => {
  const route = new RouteCodegen("GET", "/", "index");
  const debugOutput = route.emitDebug();
  expect(debugOutput).toContain("Rocket Codegen Debug [index]");
  expect(debugOutput).toContain("path: \"/\"");
});

test("CatcherCodegen and catchers helper", () => {
  const catcher404 = new CatcherCodegen(404, "not_found");
  const meta404 = catcher404.generate();
  expect(meta404.code).toBe(404);
  expect(meta404.name).toBe("not_found");
  expect(meta404.isDefault).toBe(false);

  const defaultCatcher = new CatcherCodegen(null, "default_catcher");
  const metaDefault = defaultCatcher.generate();
  expect(metaDefault.code).toBeUndefined();
  expect(metaDefault.isDefault).toBe(true);

  const catcherList = catchers([catcher404, defaultCatcher]);
  expect(catcherList.length).toBe(2);

  const genCatcher = generateCatcher(500, "internal_error");
  expect(genCatcher.code).toBe(500);
});

test("FormCodegen for FromForm derive simulation", () => {
  const form = new FormCodegen("UserForm");
  expect(form.structName).toBe("UserForm");

  form.addField("username", { name: "user_name" });
  form.addField("age", { defaultValue: "18" });
  form.addField("email", { uncased: true });

  const parsed = form.parseForm("user_name=alice&EMAIL=alice@example.com");
  expect(parsed.username).toBe("alice");
  expect(parsed.age).toBe("18");
  expect(parsed.email).toBe("alice@example.com");
});

test("FormFieldEnumCodegen for FromFormField derive simulation", () => {
  const enumCodegen = new FormFieldEnumCodegen("Color");
  expect(enumCodegen.enumName).toBe("Color");

  enumCodegen.addVariant("Red");
  enumCodegen.addVariant("Blue", ["azure", "cyan"]);

  expect(enumCodegen.parseValue("red")).toBe("Red");
  expect(enumCodegen.parseValue("AZURE")).toBe("Blue");
  expect(() => enumCodegen.parseValue("green")).toThrow();
});

test("ResponderCodegen for Responder derive simulation", () => {
  const responder = new ResponderCodegen("CustomResponse", {
    status: 404,
    contentType: "application/json",
  });
  expect(responder.typeName).toBe("CustomResponse");

  responder.addHeader("X-Custom-Header", "Rocket");
  const output = responder.respond('{"error":"not found"}');

  expect(output.status).toBe(404);
  expect(output.contentType).toBe("application/json");
  expect(output.headers["X-Custom-Header"]).toBe("Rocket");
  expect(output.body).toBe('{"error":"not found"}');
});

test("UriCodegen and generateUri for uri! macro", () => {
  const uriGen = new UriCodegen("/person/<name>?<age>");
  const uriStr = uriGen.build(
    { name: "Mike Smith", age: "28" },
    { prefix: "/api" },
  );

  expect(uriStr).toBe("/api/person/Mike%20Smith?age=28");

  const simpleUri = generateUri("/ignore/<item>", { item: "hello" });
  expect(simpleUri).toBe("/ignore/hello");
});

test("RocketApp mounting and launching", () => {
  const app = new RocketApp();
  const route = new RouteCodegen("GET", "/ping", "ping");
  const catcher = new CatcherCodegen(500, "server_error");

  app.mount("/v1", [route]);
  app.catch([catcher]);

  const routesSum = app.routesSummary();
  expect(routesSum.length).toBe(1);
  expect(routesSum[0]!.path).toBe("/v1/ping");

  const catchersSum = app.catchersSummary();
  expect(catchersSum.length).toBe(1);
  expect(catchersSum[0]!.code).toBe(500);

  const launchMsg = app.launch();
  expect(launchMsg).toContain("Rocket web framework initialized");
});

test("Codegen debug toggle", () => {
  setCodegenDebug(true);
  expect(isCodegenDebugEnabled()).toBe(true);
  setCodegenDebug(false);
  expect(isCodegenDebugEnabled()).toBe(false);
});
