FROM rust:1.82.0

# Accept build arguments
ARG OPSML_STORAGE_URI
ARG AWS_DEFAULT_REGION
ARG AWS_ACCESS_KEY_ID
ARG AWS_SECRET_ACCESS_KEY
ARG AWS_SESSION_TOKEN

# Set environment variables
ENV OPSML_STORAGE_URI=${OPSML_STORAGE_URI}
ENV AWS_DEFAULT_REGION=${AWS_DEFAULT_REGION}
ENV AWS_ACCESS_KEY_ID=${AWS_ACCESS_KEY_ID}
ENV AWS_SECRET_ACCESS_KEY=${AWS_SECRET_ACCESS_KEY}
ENV AWS_SESSION_TOKEN=${AWS_SESSION_TOKEN}

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