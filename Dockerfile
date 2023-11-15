# Use the official Rust image.
FROM rust:bookworm as builder

# Copy the manifests
RUN USER=root cargo new rustgpt
WORKDIR /rustgpt
COPY rust-toolchain.toml ./
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# This build step will cache the dependencies
RUN cargo build --release

# Now that dependencies are cached, copy the source code
COPY src src
COPY .sqlx .sqlx
# Ensure the mtimes of the source files are updated
RUN touch src/*.rs
# Build the application
RUN cargo build --release
RUN ls -lah target/release


RUN curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-arm64 && chmod +x tailwindcss-linux-arm64 && mv tailwindcss-linux-arm64 tailwindcss
COPY tailwind.config.js .
COPY input.css .
RUN ./tailwindcss -i input.css -o assets/output.css --minify


# Final base
FROM debian:bookworm-slim

RUN apt update \
    && apt install -y openssl ca-certificates \
    && apt clean \
    && rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*

# Copy the build artifact from the build stage
COPY --from=builder /rustgpt/target/release/rustgpt .
COPY templates templates
COPY assets assets
COPY db/migrations migrations

# Set the startup command to run your binary
CMD ["./rustgpt"]
