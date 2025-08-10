# Repository Guidelines

## Project Structure & Module Organization
- `backend/` (Rust + Axum): core API in `src/` with `handlers/`, `routes/`, `middleware/`, `models/`, `services/`, `utils/`, and config in `config.rs`, DB in `database.rs`, entry at `main.rs`. Migrations in `migrations/`.
- `frontend/` (SvelteKit + TS): app code in `src/` with routes and components; static assets in `static/`.
- Root scripts and docs: `test_system.sh` (end‑to‑end checks), `docker-compose.yml`, `README.md`, `API_DOCUMENTATION.md`, `DEVELOPMENT_SETUP.md`, SQL helpers (`create_*.sql`).

## Build, Test, and Development Commands
- Backend (Rust):
  - `cd backend && cargo run`: start API at `:3000`.
  - `cargo build`: compile; `cargo test`: run Rust tests.
  - `sqlx migrate run`: apply DB migrations (ensure `DATABASE_URL`).
- Frontend (SvelteKit):
  - `cd frontend && npm install && npm run dev`: start dev server at `:5173`.
  - `npm run build` / `npm run preview`: production build/preview.
  - `npm run check` / `npm run lint` / `npm run format`: typecheck, lint, format.
- Docker (full stack): `docker-compose up --build -d` then visit `http://localhost:5173`.
- System smoke test: `./test_system.sh` (checks API, DB, Redis, SSE, and key pages).

## Coding Style & Naming Conventions
- Rust: 4‑space indent; modules/files use `snake_case` (e.g., `faculty_admin.rs`); types `PascalCase`; functions/vars `snake_case`. Use `cargo fmt` and `cargo clippy` before pushing.
- Svelte/TS: 2‑space indent via Prettier; components `PascalCase.svelte`; variables/functions `camelCase`; constants `SCREAMING_SNAKE_CASE`. Run `npm run lint` and `npm run format`.

## Testing Guidelines
- Backend: prefer unit tests near code (seen as `backend/src/test_*.rs`) and handler/service tests. Run with `cargo test`. For integration, use `test_system.sh` or `curl` flows in `DEVELOPMENT_SETUP.md`.
- Frontend: no test runner configured; use `npm run check` and `npm run lint` to guard changes. If adding tests, align with SvelteKit + Vitest convention under `frontend/src`.

## Commit & Pull Request Guidelines
- Commits: concise, imperative Thai or English summaries; one logical change per commit; reference issues (e.g., `#123`) when relevant.
- PRs: include scope/goal, implementation notes, test steps, linked issues, and screenshots/GIFs for UI. Update docs (`README.md`, API/Setup) when behavior or endpoints change.

## Security & Configuration
- Do not commit secrets. Use `backend/.env` (template: `backend/.env.example`) and a local `frontend/.env`. Rotate `SESSION_SECRET` and change default admin password. Ports: API `3000`, frontend `5173`.

