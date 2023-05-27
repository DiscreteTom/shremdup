#! /bin/bash

# this file is used in github actions to install the dependencies
# according to https://github.com/rust-build/rust-build.action/tree/master

wget https://github.com/protocolbuffers/protobuf/releases/download/v23.2/protoc-23.2-linux-x86_64.zip
unzip protoc-23.2-linux-x86_64.zip -d protoc3
mv protoc3/bin/* /usr/local/bin/
mv protoc3/include/* /usr/local/include/
