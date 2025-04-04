# Use the official Rust image as the base image
FROM rust:latest AS builder

# Set the working directory inside the container
WORKDIR /usr/src/myapp

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application
RUN cargo build --release

# Use a minimal base image for the final stage
FROM debian:bookworm-slim

# Install OpenSSL
RUN apt-get update && apt-get install -y libssl-dev libpq5 ca-certificates && rm -rf /var/lib/apt/lists/* libpq-dev

# Set the working directory inside the container
WORKDIR /usr/src/myapp

# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/myapp/target/release/mockchain-engine .

# Expose the port that the application will run on
EXPOSE 8080
EXPOSE 9001

# Set the entrypoint to run the application
CMD ["./mockchain-engine"]