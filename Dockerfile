# Use the official Rust image.
FROM rustlang/rust:nightly-bookworm as builder


# Create a new empty shell project
# RUN USER=root cargo new rustgpt-app
WORKDIR /rustgpt-app

# # Copy our manifests
# COPY ./Cargo.lock ./Cargo.lock
# COPY ./Cargo.toml ./Cargo.toml

# # This build step will cache your dependencies
# RUN cargo build --release
# RUN rm src/*.rs

# Now that dependencies are cached, copy your source code
COPY . .

# Build your application
RUN cargo build --release

RUN curl -sLO https://github.com/tailwindlabs/tailwindcss/releases/latest/download/tailwindcss-linux-arm64 && chmod +x tailwindcss-linux-arm64 && mv tailwindcss-linux-arm64 tailwindcss
COPY tailwind.config.js .
COPY input.css .
RUN ./tailwindcss -i input.css -o assets/output.css --minify

RUN ls -lh target/release
RUN lll

# Final base
FROM debian:buster-slim

# Copy the build artifact from the build stage
COPY --from=builder /rustgpt-app/target/release/rustgpt .
COPY db db
COPY templates templates
COPY assets assets

# Set the startup command to run your binary
CMD ["./rustgpt"]
