# Builder
FROM ubuntu:20.04 AS rs-build

RUN apt-get update && \
    apt-get upgrade -y
RUN apt-get install libssl-dev -y

RUN apt-get install -y build-essential curl
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN rustup toolchain install 1.84.1
RUN rustup default 1.84.1

RUN rustup target add x86_64-unknown-linux-musl && \
    apt update && \
    apt install -y musl-tools musl-dev && \
    update-ca-certificates

COPY ./src ./src
COPY ./templates ./templates
COPY ./assets ./assets
COPY ./bin ./bin
COPY ./Cargo.toml .
COPY ./Cargo.lock .

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 1001 \
    "blog"

RUN cargo build --target x86_64-unknown-linux-musl --release

FROM ubuntu:20.04
RUN apt-get update -y

COPY --from=rs-build /etc/passwd /etc/passwd
COPY --from=rs-build /etc/group /etc/group

USER blog:blog

COPY --from=rs-build --chown=blog:blog ./target/x86_64-unknown-linux-musl/release/ /usr/local/bin/
COPY --from=rs-build --chown=blog:blog ./assets /usr/local/bin/assets

EXPOSE 8080

WORKDIR /usr/local/bin
CMD ["blog"]

