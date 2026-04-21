# Kyss — Finn din neste reise 🚌

A bus/transit schedule app for finding your next journey in Norway, built with Rust.

## Architecture

- **Frontend**: Leptos 0.8 (Rust → WASM, client-side rendering)
- **BFF**: Axum 0.8 (Rust HTTP server, proxies EnTur APIs)
- **Shared**: Common types crate between frontend and BFF
- **Data**: Browser localStorage with versioned JSON (future: export/share)

## Features

- 🔍 Search stops with autocomplete (EnTur Geocoder)
- 🚆 Journey planning between stops (EnTur Journey Planner)
- 💾 Save frequent searches as "Trip Types" (e.g., To work, From work)
- 🎨 Clean, responsive, mobile-first UI
- ⏱️ Real-time delay indicators
- 📦 All user data in localStorage

## Prerequisites

- Rust 1.88+ with `wasm32-unknown-unknown` target
- [Trunk](https://trunkrs.dev/) (`cargo install trunk`)

## Development

Start the BFF and frontend dev server in separate terminals:

```bash
# Terminal 1: BFF server (port 3001)
cargo run -p kyss-bff

# Terminal 2: Frontend dev server with hot reload (port 8080)
cd frontend && trunk serve
```

The frontend dev server proxies `/api/` requests to the BFF.

## Production Build

```bash
# Build frontend WASM bundle
cd frontend && trunk build --release

# Run BFF (serves frontend from dist/ and API)
cd ../bff && cargo run --release
```

Access the app at `http://localhost:3001`.

## Project Structure

```
kyss/
├── shared/          # Shared models (Stop, TripType, JourneyResult, etc.)
├── bff/             # Axum BFF server
│   └── src/
│       ├── entur/   # EnTur API clients (Geocoder, Journey Planner)
│       └── routes/  # API endpoints
├── frontend/        # Leptos WASM app
│   └── src/
│       ├── components/  # Reusable UI components
│       ├── pages/       # Route pages (Home, Search, TripType)
│       └── storage.rs   # localStorage persistence
```

## EnTur APIs

Uses free public APIs from [EnTur](https://developer.entur.org/):
- **Geocoder**: Stop search/autocomplete
- **Journey Planner**: GraphQL trip planning

All requests include `ET-Client-Name: kyss-app` header as required.
