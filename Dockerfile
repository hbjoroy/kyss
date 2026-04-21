# ─────────────────────────────────────────────────────────────────
# Multi-stage Dockerfile for Kyss (BFF + WASM frontend).
#
# Cross-compiles natively using cargo-zigbuild — no QEMU.
# The build stages always run on the host platform; only the
# final runtime image is the target arch.
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

# ── Stage 2: build BFF binary (cross-compile via zigbuild) ──────
FROM --platform=$BUILDPLATFORM docker.io/library/rust:1-bookworm AS bff-builder

ARG TARGETARCH

# Install zig (from official release) + cargo-zigbuild
RUN curl -sSL https://ziglang.org/download/0.13.0/zig-linux-$(uname -m)-0.13.0.tar.xz | tar -xJ -C /usr/local \
 && ln -s /usr/local/zig-linux-$(uname -m)-0.13.0/zig /usr/local/bin/zig \
 && cargo install cargo-zigbuild --locked

RUN rustup target add aarch64-unknown-linux-gnu x86_64-unknown-linux-gnu

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
      cargo zigbuild --release --target aarch64-unknown-linux-gnu -p kyss-bff 2>&1 || true; \
    else \
      cargo zigbuild --release --target x86_64-unknown-linux-gnu -p kyss-bff 2>&1 || true; \
    fi

# Copy real sources and build BFF
COPY shared/src shared/src
COPY bff/src bff/src

RUN touch shared/src/lib.rs && find bff/src -name '*.rs' -exec touch {} + \
 && if [ "$TARGETARCH" = "arm64" ]; then \
      cargo zigbuild --release --target aarch64-unknown-linux-gnu -p kyss-bff \
      && cp target/aarch64-unknown-linux-gnu/release/kyss-bff /binary; \
    else \
      cargo zigbuild --release --target x86_64-unknown-linux-gnu -p kyss-bff \
      && cp target/x86_64-unknown-linux-gnu/release/kyss-bff /binary; \
    fi

# ── Stage 3: extract ca-certificates on build platform ──────────
FROM --platform=$BUILDPLATFORM docker.io/library/debian:bookworm-slim AS certs
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

# ── Stage 4: minimal runtime image (target arch) ────────────────
FROM docker.io/library/debian:bookworm-slim

COPY --from=certs /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt
COPY --from=bff-builder /binary /usr/local/bin/kyss-bff
COPY --from=frontend-builder /src/frontend/dist /app/dist

ENV FRONTEND_DIST_DIR=/app/dist
ENV LISTEN_PORT=8080

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/kyss-bff"]
