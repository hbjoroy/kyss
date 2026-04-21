# Kyss — Finn din neste reise 🚌

A bus/transit schedule app for finding your next journey in Norway, built with Rust.

> **🤖 AI-assisted development**: This project was developed with the assistance of AI (GitHub Copilot). Code, architecture, and documentation were co-authored by a human and an AI pair programmer.

## Architecture

- **Frontend**: Leptos 0.8 (Rust → WASM, client-side rendering)
- **BFF**: Axum 0.8 (Rust HTTP server, proxies EnTur APIs)
- **Shared**: Common types crate between frontend and BFF
- **Data**: Browser localStorage with versioned JSON (future: export/share)

## Features

- 🔍 Search stops with autocomplete
- 🚆 Journey planning between stops with real-time data
- ⏰ Smart time period picker (morning, afternoon, evening, etc.)
- 🔄 Configurable minimum transfer gap between connections
- 💾 Save frequent searches as "Trip Types" (e.g., To work, From work)
- 🎨 Clean, responsive, mobile-first UI
- ⏱️ Real-time delay indicators
- 📦 All user data in localStorage

## EnTur API Dependency

This application depends on the free public APIs provided by [EnTur](https://developer.entur.org/):

- **[Geocoder API](https://developer.entur.org/pages-geocoder-api)**: Stop search and autocomplete
- **[Journey Planner API](https://developer.entur.org/pages-journeyplanner-journeyplanner)**: GraphQL-based trip planning with real-time data

All API requests include the `ET-Client-Name: kyss-app` header as required by EnTur's terms of use. The APIs are free and open, but rate-limited. No API key is required.

EnTur is the national registry for public transport data in Norway. Their APIs provide access to schedules, real-time data, and stop information for all public transport operators in Norway.

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

## License

MIT
