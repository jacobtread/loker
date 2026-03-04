# Debian builder image
FROM debian:trixie AS builder

# Github release version
ARG GITHUB_RELEASE_VERSION

# Docker image arch
ARG TARGETARCH

# Setup working directory
WORKDIR /app

# Install necessary tools
RUN apt-get update && \
    apt-get install -y --no-install-recommends curl ca-certificates xz-utils && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Determine binary based on arch
RUN if [ "$TARGETARCH" = "amd64" ]; then \
    ARCHIVE="loker-x86_64-unknown-linux-gnu.tar.xz"; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
    ARCHIVE="loker-aarch64-unknown-linux-gnu.tar.xz"; \
    else \
    echo "Unsupported architecture: $TARGETARCH" && exit 1; \
    fi && \
    # Download loker binary
    curl -L -o /app/loker.tar.xz https://github.com/jacobtread/loker/releases/download/${GITHUB_RELEASE_VERSION}/$ARCHIVE \
    && mkdir /app/loker-unpacked \
    && tar xf "/app/loker.tar.xz" --strip-components 1 -C "/app/loker-unpacked" \
    && mv /app/loker-unpacked/loker /app/loker \
    && chmod +x /app/loker \
    && rm -rf /app/loker-unpacked /app/locker.tar.xz


# Hardened Debian Base runner image
FROM dhi.io/debian-base:trixie

WORKDIR /app

COPY --from=builder /app/loker /app/loker

EXPOSE 8080

CMD ["/app/loker"]
