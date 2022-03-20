FROM rust:1.59.0-alpine3.14

COPY . /src/
RUN apk add --no-cache musl-dev openssl-dev openldap-dev \
	&& cd /src \
	&& cargo build --release --locked


FROM alpine:3.14

COPY --from=0 /src/target/release/barrier-backend /usr/bin/
COPY ./config.toml /opt/config.toml

RUN apk add --no-cache ca-certificates

ENV RUST_LOG="debug"

CMD [ "/usr/bin/barrier-backend", "/opt/config.toml" ]
