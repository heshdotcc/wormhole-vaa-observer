FROM rust:1.84-slim-bullseye AS builder

# Install protobuf dependencies
RUN apt-get update && apt-get install -y \
  protobuf-compiler \
  libprotobuf-dev \
  pkg-config \
  libssl-dev \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

# Dummy source to cache dependencies
RUN mkdir src && \
  echo "fn main() {}" > src/main.rs

COPY Cargo.toml Cargo.lock ./

RUN cargo build --release

COPY proto ./proto/
COPY build.rs ./

# Remove dummy source
RUN rm -rf src/

COPY src ./src/

RUN cargo build --release

FROM debian:bullseye-slim AS runtime

RUN apt-get update && apt-get install -y \
  ca-certificates \
  libssl1.1 \
  && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/app/target/release/wormhole-vaa-observer-backend ./

EXPOSE 3000

ENV RUST_LOG=info,wormhole_vaa_observer_backend::domain::wormhole::handler=debug
ENV HOST=0.0.0.0
ENV PORT=3000

CMD ["./wormhole-vaa-observer-backend"]