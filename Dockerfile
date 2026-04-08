# --- Build stage ---
FROM rust:1-alpine AS builder

RUN apk add --no-cache musl-dev gcc perl make

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY core/ core/
COPY server/ server/

# Build for the native musl target (Alpine is already musl)
RUN cargo build --release -p reddit-toxicity-server

# --- Runtime stage: scratch = ~0 bytes overhead ---
FROM scratch

COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/
COPY --from=builder /app/target/release/reddit-toxicity /reddit-toxicity

ENV HOST=0.0.0.0
ENV PORT=3000

EXPOSE 3000

ENTRYPOINT ["/reddit-toxicity"]
