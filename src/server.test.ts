import { expect, test } from "vite-plus/test";
import {
  serve,
  createFileResponse,
  createRedirectResponse,
} from "./main.ts";

test("serve with default options", () => {
  const server = serve();
  expect(server.port).toBeGreaterThan(0);
  expect(server.hostname).toBe("0.0.0.0");
  expect(server.url).toContain("http://localhost:");
  expect(server.development).toBe(false);
  server.stop();
});

test("serve with custom port and hostname", () => {
  const server = serve({
    port: 8080,
    hostname: "mydomain.com",
    development: true,
  });
  expect(server.port).toBe(8080);
  expect(server.hostname).toBe("mydomain.com");
  expect(server.url).toBe("http://mydomain.com:8080");
  expect(server.development).toBe(true);
  server.stop();
});

test("serve with random port 0", () => {
  const server = serve({ port: 0 });
  expect(server.port).toBeGreaterThan(0);
  expect(server.port).not.toBe(3000);
  server.stop();
});

test("serve with Unix domain socket", () => {
  const server = serve({ unix: "/tmp/my-socket.sock" });
  expect(server.port).toBe(0);
  expect(server.hostname).toBe("/tmp/my-socket.sock");
  expect(server.url).toBe("unix:///tmp/my-socket.sock");
  server.stop();
});

test("serve with Abstract namespace socket", () => {
  const server = serve({ unix: "\0my-abstract-socket" });
  expect(server.port).toBe(0);
  expect(server.hostname).toBe("abstract:my-abstract-socket");
  expect(server.url).toBe("unix://abstract:my-abstract-socket");
  server.stop();
});

test("Route matching (static, dynamic params, wildcard)", () => {
  const server = serve({ port: 0 });

  server.addRoute("/api/status", "OK Response");
  server.addRoute("/users/:id", "User Handler");
  server.addRoute("/api/*", "Api Fallback Handler");

  // 1. Static match
  const match1 = server.matchRoute("/api/status");
  expect(match1.matched).toBe(true);
  expect(match1.routePattern).toBe("/api/status");

  // 2. Dynamic parameter match
  const match2 = server.matchRoute("/users/12345");
  expect(match2.matched).toBe(true);
  expect(match2.routePattern).toBe("/users/:id");
  expect(match2.params.id).toBe("12345");

  // 3. Wildcard match
  const match3 = server.matchRoute("/api/v1/posts");
  expect(match3.matched).toBe(true);
  expect(match3.routePattern).toBe("/api/*");

  // 4. Unmatched route
  const match4 = server.matchRoute("/unmatched");
  expect(match4.matched).toBe(false);

  server.stop();
});

test("createFileResponse helper", () => {
  const htmlFile = createFileResponse("./index.html");
  expect(htmlFile.path).toBe("./index.html");
  expect(htmlFile.mimeType).toBe("text/html");

  const iconFile = createFileResponse("./favicon.ico");
  expect(iconFile.mimeType).toBe("image/x-icon");
});

test("createRedirectResponse helper", () => {
  const redirect = createRedirectResponse("/blog/hello/world", 301);
  expect(redirect.status).toBe(301);
  expect(redirect.headers["Location"]).toBe("/blog/hello/world");
});
