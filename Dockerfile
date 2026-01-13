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
        pipx \
        ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

ENV PATH="/root/.local/bin:${PATH}"
ENV RIA_AUTO_UPDATE=1
ENV RIA_DOWNLOAD_DIR=/tmp
WORKDIR /data
COPY --from=builder /app/target/release/riaaudio-rs /usr/bin/riaaudio-rs
CMD ["/usr/bin/riaaudio-rs"]