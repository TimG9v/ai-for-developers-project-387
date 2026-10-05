# --- backend builder ----------------------------------------------
# Теги сверены 2026-10-01 (docker manifest inspect): оба образа — bookworm,
# glibc-совместимы (бинарник cargo собирается и запускается на одной ветке
# Debian). При смене ветки — менять в ОБЕИХ стадиях.
FROM rust:1.98.1-slim-bookworm AS backend-builder
WORKDIR /build
COPY backend/Cargo.toml backend/Cargo.lock ./backend/
COPY backend/build.rs ./backend/
COPY backend/src ./backend/src
COPY openapi/openapi.json ./openapi/openapi.json
RUN cd backend && cargo build --release --locked

# --- frontend builder ----------------------------------------------
FROM node:25.2.0-bookworm-slim AS frontend-builder
WORKDIR /build/frontend
COPY frontend/package.json frontend/package-lock.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

# --- runtime --------------------------------------------------------
FROM node:25.2.0-bookworm-slim
WORKDIR /app
ENV NODE_ENV=production
# PORT не задаём: его передаёт платформа/проверка при `docker run -e PORT`.
COPY --from=backend-builder /build/backend/target/release/backend \
     /app/backend/calendar-backend
COPY --from=frontend-builder /build/frontend/node_modules \
     /app/frontend/node_modules
COPY --from=frontend-builder /build/frontend/.next /app/frontend/.next
COPY --from=frontend-builder /build/frontend/package.json \
     /app/frontend/package.json
COPY docker/entrypoint.mjs /app/entrypoint.mjs
WORKDIR /app/frontend
EXPOSE 3000
ENTRYPOINT ["node", "/app/entrypoint.mjs"]
