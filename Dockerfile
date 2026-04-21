# ─────────────────────────────────────────────────────────────────
# Multi-stage Dockerfile for Kyss (BFF + WASM frontend).
#
# Cross-compiles on the build platform — no QEMU emulation.
# Currently targets arm64 only; add amd64 via TARGETARCH later.
#
# Build context: repo root
# ─────────────────────────────────────────────────────────────────

# ── Stage 1: build frontend WASM (runs on build platform) ───────
FROM --platform=$BUILDPLATFORM docker.io/library/rust:1-bookworm AS frontend-builder

RUN rustup target add wasm32-unknown-unknown \
 && cargo install trunk --locked

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY shared/Cargo.toml shared/
COPY bff/Cargo.toml bff/
COPY frontend/Cargo.toml frontend/

# Dummy sources for dependency caching
RUN mkdir -p shared/src && echo "" > shared/src/lib.rs \
 && mkdir -p bff/src && echo "fn main(){}" > bff/src/main.rs \
 && mkdir -p bff/src/entur && echo "" > bff/src/entur/mod.rs \
 && mkdir -p bff/src/routes && echo "" > bff/src/routes/mod.rs \
 && mkdir -p frontend/src && echo "fn main(){}" > frontend/src/main.rs \
 && cargo build --release --target wasm32-unknown-unknown -p kyss-frontend 2>&1 || true

# Copy real sources and build frontend
COPY shared/src shared/src
COPY frontend/src frontend/src
COPY frontend/index.html frontend/
COPY frontend/Trunk.toml frontend/
COPY frontend/style frontend/style

RUN cd frontend && trunk build --release

# ── Stage 2: build BFF binary (cross-compile for target arch) ───
FROM --platform=$BUILDPLATFORM docker.io/library/rust:1-bookworm AS bff-builder

ARG TARGETARCH

RUN if [ "$TARGETARCH" = "arm64" ]; then \
      apt-get update && apt-get install -y gcc-aarch64-linux-gnu && rm -rf /var/lib/apt/lists/*; \
    elif [ "$TARGETARCH" = "amd64" ]; then \
      apt-get update && apt-get install -y gcc-x86-64-linux-gnu && rm -rf /var/lib/apt/lists/*; \
    fi

RUN rustup target add aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc
ENV CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=x86_64-linux-gnu-gcc

WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY shared/Cargo.toml shared/
COPY bff/Cargo.toml bff/
COPY frontend/Cargo.toml frontend/

# Dummy sources for dependency caching
RUN mkdir -p shared/src && echo "" > shared/src/lib.rs \
 && mkdir -p bff/src && echo "fn main(){}" > bff/src/main.rs \
 && mkdir -p bff/src/entur && echo "" > bff/src/entur/mod.rs \
 && mkdir -p bff/src/routes && echo "" > bff/src/routes/mod.rs \
 && mkdir -p frontend/src && echo "fn main(){}" > frontend/src/main.rs

RUN if [ "$TARGETARCH" = "arm64" ]; then \
      cargo build --release --target aarch64-unknown-linux-gnu -p kyss-bff 2>&1 || true; \
    else \
      cargo build --release --target x86_64-unknown-linux-gnu -p kyss-bff 2>&1 || true; \
    fi

# Copy real sources and build BFF
COPY shared/src shared/src
COPY bff/src bff/src

RUN touch shared/src/lib.rs && find bff/src -name '*.rs' -exec touch {} + \
 && if [ "$TARGETARCH" = "arm64" ]; then \
      cargo build --release --target aarch64-unknown-linux-gnu -p kyss-bff \
      && cp target/aarch64-unknown-linux-gnu/release/kyss-bff /binary; \
    else \
      cargo build --release --target x86_64-unknown-linux-gnu -p kyss-bff \
      && cp target/x86_64-unknown-linux-gnu/release/kyss-bff /binary; \
    fi

# ── Stage 3: extract ca-certificates on build platform ──────────
FROM --platform=$BUILDPLATFORM docker.io/library/debian:bookworm-slim AS certs
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

# ── Stage 4: minimal runtime image ──────────────────────────────
FROM docker.io/library/debian:bookworm-slim

COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=bff-builder /binary /usr/local/bin/kyss-bff
COPY --from=frontend-builder /src/frontend/dist /app/dist

ENV FRONTEND_DIST_DIR=/app/dist
ENV LISTEN_PORT=8080

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/kyss-bff"]
