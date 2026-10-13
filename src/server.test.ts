import { expect, test, describe } from "vite-plus/test";
import { Server } from "./main.ts";

describe("Rocket Server NAPI bindings", () => {
  test("Server creation and configuration", () => {
    const server = new Server(8080, "127.0.0.1");
    expect(server.getPort()).toBe(8080);
    expect(server.getAddress()).toBe("127.0.0.1");
  });

  test("Mounting routes and launching server", () => {
    const server = new Server(8081, "127.0.0.1");
    server.mount("/api", [
      { method: "GET", path: "/hello" },
      { method: "POST", path: "/data" },
    ]);
    server.route("GET", "/ping", "pong");

    const launchMsg = server.launch();
    expect(launchMsg).toContain("Rocket server launching on http://127.0.0.1:8081");
    server.stop();
  });
});
