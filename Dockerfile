# --- Build stage ---
FROM rust:1-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY core/ core/
COPY server/ server/

RUN cargo build --release --target x86_64-unknown-linux-musl -p reddit-toxicity-server 2>/dev/null \
    || (rustup target add x86_64-unknown-linux-musl && cargo build --release --target x86_64-unknown-linux-musl -p reddit-toxicity-server)

# --- Runtime stage: scratch = ~0 bytes overhead ---
FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/reddit-toxicity /reddit-toxicity

ENV HOST=0.0.0.0
ENV PORT=3000

EXPOSE 3000

ENTRYPOINT ["/reddit-toxicity"]
