FROM rust:1.95-slim-bookworm AS builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests and local path dependencies first for layer caching
COPY Cargo.toml Cargo.lock ./
COPY async-sqlx-session/ ./async-sqlx-session/

# Build a dummy main to cache dependencies
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
    && cargo build --release \
    && rm -rf src

# Build the real application
COPY src/ ./src/
# Touch main.rs so cargo rebuilds it (avoids stale cache from dummy)
RUN touch src/main.rs && cargo build --release

# ---- Runtime image ----
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd --gid 10002 riteapp && \
    useradd --system --uid 10002 --gid 10002 --no-create-home --shell /usr/sbin/nologin riteapp

WORKDIR /app

COPY --from=builder /app/target/release/rite-cloud ./rite-cloud
COPY templates/ ./templates/
COPY res/ ./res/

RUN chown -R riteapp:riteapp /app

USER riteapp

EXPOSE 5000

CMD ["./rite-cloud"]
