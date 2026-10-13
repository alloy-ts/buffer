import { expect, test } from "vite-plus/test";
import { RocketServer } from "../index.js";

test("RocketServer instantiation and methods", () => {
  const server = new RocketServer(8080);
  expect(server.getPort()).toBe(8080);
  expect(server.isRunning()).toBe(false);

  const defaultServer = new RocketServer();
  expect(defaultServer.getPort()).toBe(8000);
});
