import type { NextConfig } from "next";

// Прокси API на внутренний backend. Цель — константа: Next разрешает
// rewrites при `next build` (routes-manifest), runtime-переменные здесь
// не работают. Топология контейнера фиксирована entrypoint'ом (task-6,
// шаг 3): backend всегда на 127.0.0.1:8081.
const backend = "http://127.0.0.1:8081";

const nextConfig: NextConfig = {
  async rewrites() {
    return [
      { source: "/api/:path*", destination: `${backend}/:path*` },
      { source: "/health", destination: `${backend}/health` },
    ];
  },
};

export default nextConfig;
