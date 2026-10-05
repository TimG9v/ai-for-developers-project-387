.PHONY: dev dev-backend dev-frontend test test-backend test-frontend lint lint-backend lint-frontend generate stop

dev:
	@cd backend && cargo run &
	@cd frontend && npm run dev &
	@wait

dev-backend:
	cd backend && cargo run

dev-frontend:
	cd frontend && npm run dev

generate:
	cd contracts && npx tsp compile .
	cd frontend && npm run generate:client
	cd backend && cargo build

stop:
	@-fuser -k 3000/tcp 8081/tcp 2>/dev/null
	@echo "stopped backend (:8081) and frontend (:3000), if they were running"

test: test-backend test-frontend

test-backend:
	cd backend && cargo test

test-frontend:
	cd frontend && npm test

lint: lint-backend lint-frontend

lint-backend:
	cd backend && cargo fmt --check
	cd backend && cargo clippy --all-targets -- -D warnings

lint-frontend:
	cd frontend && npm run lint
