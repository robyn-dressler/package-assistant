ARG DISTRO_NAME

#Build application
FROM rust:latest as builder
WORKDIR /home/root/src
COPY . .
RUN --mount=type=cache,target=/home/rust/.cargo/git \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,sharing=private,target=/usr/src/package-assistant/target \
    cargo build && \
    cp target/debug/package-assistant ./package-assistant

#Setup test environment
FROM ${DISTRO_NAME}-frozen
WORKDIR /home/root

COPY --from=builder /home/root/src/package-assistant /usr/local/bin/package-assistant

ARG DISTRO_NAME
ARG REFRESH_REPO="echo 'not refreshing repos'"
ARG INSTALL_DEPS="echo 'no dependencies to install'"
RUN ${REFRESH_REPO}
RUN ${INSTALL_DEPS}
COPY "docker/settings-${DISTRO_NAME}.toml" settings.toml
RUN package-assistant init -c settings.toml

CMD ["package-assistant", "test"]