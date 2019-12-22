#!/bin/bash

cargo build --release --target=x86_64-unknown-linux-musl && scp target/x86_64-unknown-linux-musl/release/grease_api mensgleeclub@gleeclub.gatech.edu:/cgi-bin/api
cargo doc --no-deps && scp target/doc/grease_api/ mensgleeclub@gleeclub.gatech.edu:/httpsdocs/api-docs/
