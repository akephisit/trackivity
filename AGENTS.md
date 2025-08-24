# Repository Guidelines

This document is a concise contributor guide for the Trackivity repository.

## Project Structure & Module Organization
- `backend/` (Rust + Axum): core API in `src/` with `handlers/`, `routes/`, `middleware/`, `models/`, `services/`, `utils/`; config in `config.rs`, DB in `database.rs`, entry at `main.rs`. Migrations in `migrations/`.
- `frontend/` (SvelteKit + TypeScript): app in `src/` (routes, components); static assets in `static/`.
- Root scripts/docs: `test_system.sh` (end‑to‑end check), `docker-compose.yml`, `README.md`, `API_DOCUMENTATION.md`, `DEVELOPMENT_SETUP.md`, SQL helpers (`create_*.sql`).

## Build, Test, and Development Commands
- Backend:
  - `cd backend && cargo run` — start API on `:3000`.
  - `cargo build` — compile; `cargo test` — run Rust tests.
  - `sqlx migrate run` — apply DB migrations (requires `DATABASE_URL`).
- Frontend:
  - `cd frontend && npm install && npm run dev` — start dev server on `:5173`.
  - `npm run build` / `npm run preview` — production build/preview.
  - `npm run check` / `npm run lint` / `npm run format` — typecheck, lint, format.
- Full stack via Docker: `docker-compose up --build -d` → visit `http://localhost:5173`.
- Smoke test: `./test_system.sh` (API, DB, Redis, SSE, key pages).

## Coding Style & Naming Conventions
- Rust: 4‑space indent; modules/files `snake_case`; types `PascalCase`; vars/functions `snake_case`.
  - Run `cargo fmt` and `cargo clippy` before pushing.
- Svelte/TS: 2‑space indent via Prettier; components `PascalCase.svelte`; vars/functions `camelCase`; constants `SCREAMING_SNAKE_CASE`.
  - Run `npm run lint` and `npm run format`.

## Testing Guidelines
- Backend: prefer unit tests close to code (e.g., `backend/src/test_*.rs`) and handler/service tests. Run with `cargo test`.
- Integration: use `test_system.sh` or `curl` flows from `DEVELOPMENT_SETUP.md`.
- Frontend: no test runner configured; guard with `npm run check` and `npm run lint`.

## Commit & Pull Request Guidelines
- Commits: concise, imperative Thai or English; one logical change per commit; reference issues (e.g., `#123`) when relevant.
- PRs: include scope/goal, implementation notes, test steps, linked issues, and screenshots/GIFs for UI; update docs (`README.md`, API/setup) when behavior or endpoints change.

## Security & Configuration
- Do not commit secrets. Use `backend/.env` (template: `backend/.env.example`) and `frontend/.env` locally.
- Rotate `SESSION_SECRET` and change default admin password.
- Default ports: API `3000`, frontend `5173`.

