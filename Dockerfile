FROM ekidd/rust-musl-builder:nightly AS builder

ADD . ./

RUN sudo chown -R rust:rust /home/rust

RUN cargo build --release

FROM alpine:latest
COPY --from=builder \
    /home/rust/src/target/x86_64-unknown-linux-musl/release/pastebin \
    /usr/local/bin/

WORKDIR /app
RUN mkdir ./upload

CMD /usr/local/bin/pastebin
