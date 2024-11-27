FROM rust:1.82.0

# Accept build arguments
ARG OPSML_STORAGE_URI

# Set environment variables
ENV OPSML_STORAGE_URI=${OPSML_STORAGE_URI}

# Create app directory
WORKDIR /app

# Copy the top-level Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock /app/

# Copy the crates directory
COPY crates /app/crates

# Copy the opsml_core directory
COPY opsml_core /app/opsml_core

# Make the script executable
RUN chmod +x /app/crates/test.sh

# Set the entrypoint to the script
ENTRYPOINT ["/app/crates/test.sh"]