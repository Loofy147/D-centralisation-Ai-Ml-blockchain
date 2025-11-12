# Stage 1: Build the application
FROM rust:1.79 as builder

# Create a new empty shell project
WORKDIR /usr/src/notary_server
COPY . .

# Build the application
RUN cargo build --release

# Stage 2: Create the final, smaller image
FROM debian:bookworm-slim

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/notary_server/target/release/notary_server .

# Set the command to run the application
CMD ["./notary_server"]
