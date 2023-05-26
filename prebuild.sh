#! /bin/bash

# this file is used in github actions to install the dependencies
# according to https://github.com/rust-build/rust-build.action/tree/master

wget https://github.com/protocolbuffers/protobuf/releases/download/v3.20.3/protoc-3.20.3-linux-x86_64.zip
unzip protoc-3.20.3-linux-x86_64.zip -d protoc3
mv protoc3/bin/* /usr/local/bin/
export PROTOC=/usr/local/bin/protoc
export RUST_LOG=debug

# test the build
RUST_LOG=debug cargo build