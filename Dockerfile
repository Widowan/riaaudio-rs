# ---- Build ----
FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# ---- Runtime ----
FROM debian:13-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ffmpeg \
        python3 \
        python3-pip \
        ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

ENV RIA_AUTO_UPDATE=1
COPY --from=builder /app/target/release/riaaudio-rs /usr/bin/riaaudio-rs
CMD ["/usr/bin/riaaudio-rs"]