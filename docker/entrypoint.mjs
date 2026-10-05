import { spawn } from "node:child_process";

// Supervisor контейнера: поднимает backend и Next.js, пересылает сигналы
// и завершает контейнер, если умер любой из процессов (fail-fast вместо
// полуживого сервиса: страницы без API).
const port = process.env.PORT ?? "3000";
const backendPort = process.env.BACKEND_PORT ?? "8081";

const backend = spawn("/app/backend/calendar-backend", [], {
  stdio: "inherit",
  env: { ...process.env, BACKEND_PORT: backendPort },
});
const web = spawn(
  "node",
  [
    "node_modules/next/dist/bin/next",
    "start",
    "--hostname",
    "0.0.0.0",
    "--port",
    port,
  ],
  { cwd: "/app/frontend", stdio: "inherit" },
);

const shutdown = (signal) => {
  for (const child of [backend, web]) child.kill(signal);
};
process.on("SIGTERM", () => shutdown("SIGTERM"));
process.on("SIGINT", () => shutdown("SIGINT"));

for (const child of [backend, web]) {
  child.on("exit", (code) => process.exit(code ?? 1));
}
