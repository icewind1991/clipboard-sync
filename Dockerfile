FROM scratch

ADD target/x86_64-unknown-linux-musl/release/server /
EXPOSE 80

CMD ["/server"]