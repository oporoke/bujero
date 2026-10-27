# Bujero — Wholesale Stock Management SaaS

## Product
- Multi-tenant SaaS stock management + POS for wholesalers. Alcohol sellers first; pharmacies and general wholesale later.
- Markets: Tanzania and Kenya (TZS/KES). Prices VAT-inclusive; excise shown on receipts.
- Fiscal (TRA VFD/EFD, KRA eTIMS): NOT in MVP. Backend must be integration-ready (tax data per sale line, fiscal adapter interface).
- Flat subscription billing per tenant. Tenants can be a single shop or multi-branch.

## Core domain rules
- Stock stored in ONE base unit per product (e.g. bottle). Never store mixed units.
- Products have pack levels (bottle=1, six-pack=6, box=24, crate=20/25), each with its own price and barcode.
- Sales lines are entered in any pack level, converted to base units for stock.
- Stock is an append-only movement ledger (sale, receipt, adjustment, transfer, empties). Stock on hand = sum of movements.
- Returnable empties (crates) are in MVP: deposits charged/refunded, empties tracked per customer.
- Pricing: per pack level; optional price tiers (tenant setting).
- Credit sales: tenant setting; per-customer credit limits.

## Architecture
- Backend: Rust, Axum 0.8, Postgres 18, sqlx. Cargo workspace: crates/api (lib + thin main.rs), crates/domain, crates/shared.
- Dependency direction: api -> domain -> shared. Never reverse.
- Clients: one Angular codebase on web (PWA), Tauri 2 desktop (Windows, Linux), Tauri 2 Android.
- OFFLINE-FIRST everywhere, including web (full offline POS). Local store: SQLite (Tauri), IndexedDB (web), behind one Angular data interface.
- Sync: device-generated UUID v7 IDs, outbox push + checkpoint pull, idempotency keys, per-device receipt numbering, negative stock allowed offline and flagged on sync.
- Printing: Tauri prints ESC/POS directly (USB desktop, Bluetooth Android); web uses browser print dialog.

## Environment
- Ubuntu, Rust 1.99.1 (pinned in rust-toolchain.toml), Node 24, Docker Compose v5.
- Postgres in Docker on host port 55432 (system Postgres occupies 5433). See compose.yaml, .env.example.
- API runs on port 3000. Config from env via crates/api/src/config.rs (dotenvy for .env in dev).
- Logging: tracing + tracing-subscriber; LOG_FORMAT=pretty|json; RUST_LOG filter; request IDs via tower-http.

## Working style (IMPORTANT)
- This is also an A-Z Axum learning project. Work ONE step at a time from the build plan.
- Each step: say what it is and why, make the change, explain what each part does, then verify (build, tests, curl).
- Concise, point-form explanations. No trial and error: verify versions/APIs (docs) before using them.
- Zero compiler warnings. Every step ships with tests. Commit at the end of each step: "Step N: <summary>".
- Scope/plan doc (Claude Doc): https://claude.ai/artifact/5xVGEPdhYyWqLLBME4H64t

## Build plan
- Phase 0 Setup: 0 toolchain [done], 1 workspace [done], 2 Postgres via Compose [done]
- Phase 1 Backend foundation: 3 minimal server [done], 4 config [done], 5 tracing + request IDs [done], 6 error handling [IN PROGRESS], 7 sqlx pool + migrations, 8 app state + FromRef, 9 core middleware (CORS, compression, timeouts, body limit) + graceful shutdown, 10 test harness (oneshot, test DB)
- Phase 2 Tenancy/security: 11 tenants + branches, 12 users + argon2, 13 JWT auth + device registration, 14 custom extractors (CurrentUser, TenantContext) + rejections, 15 roles/permissions (route_layer), 16 rate limiting + security headers
- Phase 3 Catalog: 17 categories/products CRUD, 18 pack levels + unit conversion, 19 barcodes, 20 pricing (tiers, effective dates), 21 fast search (pg_trgm)
- Phase 4 Stock: 22 movement ledger, 23 suppliers + GRN, 24 adjustments + stock counts, 25 branch transfers, 26 returnable empties
- Phase 5 Sales: 27 customers + credit, 28 quotations, 29 sales, 30 payments (split, mobile money, credit), 31 per-device receipt numbers, 32 fiscal adapter (no-op), 33 cashier shifts
- Phase 6 Sync: 34 protocol design, 35 push, 36 pull, 37 conflict/negative-stock flags, 38 live updates (WS/SSE)
- Phase 7 Reports/files: 39 reports, 40 CSV/PDF export + bulk import (multipart), 41 OpenAPI (utoipa)
- Phase 8 Angular: 42 workspace, 43 data interface + IndexedDB, 44 sync engine, 45 auth, 46 catalog screens, 47 POS screen, 48 receipt + browser print, 49 quotations/customers/stock/empties, 50 reports, 51 PWA
- Phase 9 Tauri 2: 52 desktop shell, 53 SQLite data layer, 54 USB ESC/POS, 55 Android, 56 Bluetooth printing
- Phase 10 SaaS/prod: 57 signup + subscription, 58 backups, 59 deployment, 60 CI/CD + auto-update, 61 monitoring, 62 pilot, 63 launch

## Current state (Step 6)
- Done: crates/api/src/error.rs (AppError + IntoResponse, JSON {error:{code,message,details?}}), lib.rs (app() with fallback + method_not_allowed_fallback + layers), 5 lib tests pass.
- Pending: crates/api/src/main.rs still has old `mod config; mod telemetry;` — must become thin: `use api::{app, config::Config, telemetry};`. This causes a dead_code warning on database_url.
- Port 3000 may be held by an old `api` process (check `ss -ltnp | grep ':3000 '`).
- Then verify: zero warnings, curl /nope -> 404 JSON, POST /health -> 405 JSON with allow header. Commit "Step 6: error handling".
