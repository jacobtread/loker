# ----------------------------------------
#  Builder part
# ----------------------------------------
FROM rust:1.93.1-trixie-bookworm AS builder

WORKDIR /app

# Install required dependencies for openssl-sys
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    build-essential \
    pkg-config \
    perl \
    nasm \
    libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Dependency precachng
COPY Cargo.toml .
COPY Cargo.lock .

# Create empty entrypoints
RUN mkdir src
RUN echo "fn main() {}" >src/main.rs
RUN echo "//placeholder" >src/lib.rs

# Run a build to download dependencies
RUN cargo build --release

# Copy actual source
COPY src src

# Touch files to trigger rebuild
RUN touch src/main.rs
RUN touch src/lib.rs

RUN cargo build --release


# ----------------------------------------
# Runner part
# ----------------------------------------
FROM dhi.io/debian-base:trixie AS runner

WORKDIR /app

# Copy the built binary
COPY --from=builder /app/target/release/loker ./

EXPOSE 8080

CMD ["/app/loker"]
