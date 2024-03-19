FROM rust:latest as build

WORKDIR /home/app

# Copy the project files into the Docker image
COPY ./ ./

# Build the program
RUN cargo build --release \
    cargo install --path .

FROM rust:slim-buster

COPY --from=build /usr/bin/example-rust-crud-strategy-pattern /usr/bin/

# Run the program
CMD ["example-rust-crud-strategy-pattern"]
