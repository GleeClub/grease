FROM clux/muslrust:nightly as builder

# Ensure that queries are cached with `cargo sqlx prepare`
ENV SQLX_OFFLINE=true

# Make a fake Rust app to keep a cached layer of compiled crates
RUN USER=root cargo new app
WORKDIR /usr/src/app
COPY Cargo.toml Cargo.lock ./
RUN mkdir .cargo
# This is the trick to speed up the building process
RUN cargo vendor > .cargo/config

# Copy the rest
COPY . .
# Reset the vendor config
RUN cargo vendor >> .cargo/config
# Build (install) the actual binaries
RUN cargo install --path .

# Runtime image
FROM alpine:latest

# Install timezone data
RUN apk add tzdata

# Set current timezone
RUN cp /usr/share/zoneinfo/America/New_York /etc/localtime
RUN echo "America/New_York" > /etc/timezone

# Remove other timezone files
RUN apk del tzdata

# Get compiled binaries from builder's cargo install directory
COPY --from=builder /root/.cargo/bin/grease /bin