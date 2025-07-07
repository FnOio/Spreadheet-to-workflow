FROM rust:1-alpine AS rust
RUN apk add --no-cache musl-dev
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY src /src
RUN cargo build --release

FROM openjdk:18-ea-11-alpine
RUN apk add --no-cache bash nodejs npm wget && cd / && \

    ########## YARRRML-parser ##########
    npm install -g @rmlio/yarrrml-parser && \

    ############# RMLMapper ############
    wget --output-document=rmlmapper.jar https://github.com/RMLio/rmlmapper-java/releases/download/v7.3.3/rmlmapper-7.3.3-r374-all.jar && \
    mkdir /mnt/data

COPY resources resources
COPY docker/run.sh /run.sh
COPY --from=rust /target/release/spreadsheet-to-flow spreadsheet-to-flow

VOLUME ["/mnt/data"]

ENTRYPOINT ["/run.sh"]
