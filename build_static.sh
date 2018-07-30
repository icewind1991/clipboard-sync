#!/usr/bin/env sh
echo "Building static binaries using ekidd/rust-musl-builder"
docker build -t build-"$1"-image -f Dockerfile.build .
docker run -it --name build-"$1" build-"$1"-image
mkdir -p target/x86_64-unknown-linux-musl/release
docker cp build-"$1":/home/rust/src/target/x86_64-unknown-linux-musl/release/"$1" "target/x86_64-unknown-linux-musl/release/$1"
docker rm build-"$1"
docker rmi build-"$1"-image