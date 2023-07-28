# Use the official Rust base image
FROM rust:latest

# Set the working directory in the container
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files to the container
COPY Cargo.toml ./

# Copy the rest of the source code to the container
COPY . .

# Build the dependencies first (cache the dependencies)
RUN cargo build --release

# Expose the port that Actix Web listens on (change the port number if needed)
EXPOSE 8080

# Run the Actix Web application on container startup
CMD ["cargo", "run", "--release"]
