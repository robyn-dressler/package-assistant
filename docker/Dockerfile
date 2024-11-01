ARG BASE_IMAGE

#Build application
FROM rust:latest as builder
WORKDIR /usr/src/package-assistant
COPY . .
RUN cargo build --release

#Setup test environment
FROM ${BASE_IMAGE}
WORKDIR /usr/local/package-assistant

COPY --from=builder /usr/src/package-assistant/target/release/package-assistant /usr/local/bin/package-assistant

RUN useradd -m pkguser
USER pkguser

WORKDIR /home/pkguser
ARG DISTRO_NAME
COPY "docker/settings-${DISTRO_NAME}.toml" settings.toml
RUN package-assistant init -c settings.toml

CMD ["package-assistant", "test"]