#!/bin/bash

cargo build --release --target=x86_64-unknown-linux-musl && scp target/x86_64-unknown-linux-musl/release/api mensgleeclub@gleeclub.gatech.edu:/cgi-bin/api
