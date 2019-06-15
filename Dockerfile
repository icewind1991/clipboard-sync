FROM ekidd/rust-musl-builder AS build

RUN sudo apt-get update && sudo apt-get install -y xorg-dev python3

# We need to add the source code to the image because `rust-musl-builder`
# assumes a UID of 1000, but TravisCI has switched to 2000.
ADD . ./
RUN sudo chown -R rust:rust .

RUN cargo build --release --bin server

FROM scratch

COPY --from=build /home/rust/src/target/x86_64-unknown-linux-musl/release/server /
EXPOSE 80

CMD ["/server"]