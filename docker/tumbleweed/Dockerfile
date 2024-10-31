#Build application
FROM rust:latest as builder
WORKDIR /usr/src/package-assistant
COPY . .
RUN cargo build --release

#Setup test environment
FROM opensuse/tumbleweed:latest
WORKDIR /usr/local/package-assistant

COPY --from=builder /usr/src/package-assistant/target/release/package-assistant /usr/local/bin/package-assistant

#Setup cached packages
RUN zypper dup -d -y

RUN useradd -m pkguser
USER pkguser

WORKDIR /home/pkguser
COPY docker/tumbleweed/settings.toml .
RUN package-assistant init -c settings.toml