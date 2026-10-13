import { expect, test } from "vite-plus/test";
import * as native from "../index.js";
import {
  RocketServer,
  createResponse,
  createJsonResponse,
  HttpMethod,
} from "../index.js";

test("RocketServer - initialization and configuration", () => {
  const server = RocketServer.build();
  expect(server.port).toBe(8000);
  expect(server.address).toBe("127.0.0.1");

  server.port = 9000;
  expect(server.port).toBe(9000);

  const customServer = RocketServer.custom({
    port: 3000,
    address: "0.0.0.0",
  });
  expect(customServer.port).toBe(3000);
  expect(customServer.address).toBe("0.0.0.0");
});

test("RocketServer - simple route mounting and dispatching", async () => {
  const server = RocketServer.build();

  server.mountSimpleRoute("/", "GET", "/", "Hello from Rocket!", 200);
  server.mountSimpleRoute("/api", "POST", "/hello", "Hello API!", 201);

  expect(server.routesCount).toBe(2);

  const res1 = await server.dispatch("GET", "/", null);
  expect(res1.status).toBe(200);
  expect(res1.body ? res1.body.toString("utf8") : "").toBe("Hello from Rocket!");

  const res2 = await server.dispatch("POST", "/api/hello", null);
  expect(res2.status).toBe(201);
  expect(res2.body ? res2.body.toString("utf8") : "").toBe("Hello API!");
});

test("RocketServer - JS route handler with parameters and JSON response", async () => {
  const server = RocketServer.build();

  server.mountRoute(
    "/user",
    {
      method: "GET",
      path: "/<id>",
    },
    (_err, req) => {
      return createJsonResponse(
        200,
        JSON.stringify({ userId: req.params.id || "unknown", status: "active" })
      );
    }
  );

  const res = await server.dispatch("GET", "/user/42", null);
  expect(res.status).toBe(200);
  expect(res.contentType).toContain("application/json");
  const bodyText = res.body ? res.body.toString("utf8") : "";
  expect(JSON.parse(bodyText)).toEqual({ userId: "42", status: "active" });
});

test("Response helpers - createResponse and createJsonResponse", () => {
  const res1 = createResponse(200, "Plain Text", "text/plain", { "X-Custom": "test" });
  expect(res1.status).toBe(200);
  expect(res1.contentType).toBe("text/plain");
  expect(res1.headers?.["X-Custom"]).toBe("test");

  const res2 = createJsonResponse(201, '{"created":true}');
  expect(res2.status).toBe(201);
  expect(res2.contentType).toBe("application/json");
});

test("HttpMethod enum mapping", () => {
  expect(native.HttpMethod).toBeDefined();
});
