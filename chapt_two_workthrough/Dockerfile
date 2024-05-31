# syntax=docker/dockerfile:1

# Define the base Rust version and the application name as build arguments
ARG RUST_VERSION=1.77.2
ARG APP_NAME=chapt_two_workthrough

################################################################################
# Build stage
FROM rust:${RUST_VERSION}-alpine AS build
ARG APP_NAME

# Set the working directory
WORKDIR /app

# Install required build dependencies
RUN apk add --no-cache clang lld musl-dev git

# Copy the entire application directory into the container
COPY . .

# Build the application
RUN cargo build --locked --release

################################################################################
# Runtime stage
FROM alpine:3.18 AS final

# Define a non-privileged user to run the application
ARG UID=10001
RUN adduser --disabled-password --gecos "" --home "/nonexistent" --shell "/sbin/nologin" --no-create-home --uid "${UID}" appuser
USER appuser

# Set the working directory
WORKDIR /app

# Copy the executable from the build stage
COPY --from=build /app/target/release/$APP_NAME /bin/server

# Copy application assets and migrations
COPY --chown=appuser:appuser ./assets ./assets
COPY --chown=appuser:appuser ./migrations ./migrations

# Expose the port the application listens on
EXPOSE 7000

# Ensure the secrets directory exists and the secrets file is copied
# COPY --chown=appuser:appuser /run/secrets/db-password /run/secrets/db-password

# Command to run the application
CMD ["/bin/server/chapt_two_workthrough"]