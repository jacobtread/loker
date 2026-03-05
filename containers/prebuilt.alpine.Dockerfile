# Alpine builder image
FROM alpine:3.23 AS builder

# Github release version
ARG GITHUB_RELEASE_VERSION

# Docker image arch
ARG TARGETARCH

# Setup working directory
WORKDIR /app

# Install necessary tools
RUN apk add --no-cache curl ca-certificates xz

# Determine binary based on arch
RUN if [ "$TARGETARCH" = "amd64" ]; then \
    ARCHIVE="loker-x86_64-unknown-linux-musl.tar.xz"; \
    elif [ "$TARGETARCH" = "arm64" ]; then \
    ARCHIVE="loker-aarch64-unknown-linux-musl.tar.xz"; \
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

# Alpine runner image
FROM alpine:3.23

WORKDIR /app

ENV SM_DATABASE_PATH=/data/secrets.db

VOLUME ["/data"]

COPY --from=builder /app/loker /app/loker

EXPOSE 8080

CMD ["/app/loker"]
