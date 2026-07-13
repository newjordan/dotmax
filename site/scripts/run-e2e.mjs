import { spawn } from "node:child_process";
import { setTimeout as delay } from "node:timers/promises";

const host = process.env.E2E_HOST ?? "127.0.0.1";
const port = process.env.E2E_PORT ?? "4173";
// E2E_CMD=preview serves the built dist/ (vite preview) instead of the dev
// server; E2E_BASE=/dotmax/ additionally serves it under a subpath, which is
// how the GitHub Pages smoke test (tests/base-path.spec.ts) runs.
const command = process.env.E2E_CMD === "preview" ? "preview" : "dev";
let basePath = process.env.E2E_BASE ?? "";
if (basePath && !basePath.endsWith("/")) basePath += "/";
const baseURL = `http://${host}:${port}${basePath || ""}`;
const isWindows = process.platform === "win32";
const viteBin = `node_modules/.bin/vite${isWindows ? ".cmd" : ""}`;
const playwrightBin = `node_modules/.bin/playwright${isWindows ? ".cmd" : ""}`;

let server;
let serverExited = true;
let shuttingDown = false;

function runCommand(command, args) {
  return new Promise((resolve) => {
    const child = spawn(command, args, {
      stdio: "inherit",
      env: process.env,
    });

    child.on("exit", (code) => resolve(code ?? 1));
  });
}

function startServer() {
  serverExited = false;
  const args =
    command === "preview"
      ? ["preview", "--host", host, "--port", port, "--strictPort"]
      : ["--host", host, "--port", port, "--strictPort"];
  if (basePath) args.push("--base", basePath);
  server = spawn(viteBin, args, {
    stdio: ["ignore", "pipe", "pipe"],
    env: process.env,
  });

  server.stdout.on("data", (chunk) => process.stdout.write(chunk));
  server.stderr.on("data", (chunk) => process.stderr.write(chunk));
  server.on("exit", (code, signal) => {
    serverExited = true;
    if (shuttingDown) return;

    if (code !== 0 && code !== null) {
      process.stderr.write(`Vite exited before tests completed with code ${code}\n`);
    } else if (signal && signal !== "SIGTERM") {
      process.stderr.write(`Vite exited before tests completed from signal ${signal}\n`);
    }
  });
}

async function waitForServer() {
  const deadline = Date.now() + 120_000;

  while (Date.now() < deadline) {
    if (serverExited) {
      throw new Error("Vite exited before it became available.");
    }

    try {
      const response = await fetch(baseURL);
      if (response.ok) return;
    } catch {
      // Keep polling until Vite opens the port.
    }

    await delay(250);
  }

  throw new Error(`Timed out waiting for ${baseURL}`);
}

function runPlaywright() {
  return new Promise((resolve) => {
    const testEnv = {
      ...process.env,
      PLAYWRIGHT_BASE_URL: baseURL,
    };
    delete testEnv.FORCE_COLOR;

    const args = process.env.BASE_PATH_E2E ? ["test", "tests/base-path.spec.ts"] : ["test"];
    const testProcess = spawn(playwrightBin, args, {
      stdio: "inherit",
      env: testEnv,
    });

    testProcess.on("exit", (code) => resolve(code ?? 1));
  });
}

try {
  const installExitCode = await runCommand(playwrightBin, ["install", "chromium"]);
  if (installExitCode !== 0) {
    throw new Error(`playwright install chromium failed with exit code ${installExitCode}`);
  }

  startServer();
  await waitForServer();
  const exitCode = await runPlaywright();
  process.exitCode = exitCode;
} catch (error) {
  process.stderr.write(`${error instanceof Error ? error.message : String(error)}\n`);
  process.exitCode = 1;
} finally {
  if (server && !serverExited) {
    shuttingDown = true;
    server.kill("SIGTERM");
  }
}
