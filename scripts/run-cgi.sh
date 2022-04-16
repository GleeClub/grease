#!/bin/bash

# Build `grease`, put it in the `cgi-bin/` folder,
# and run CGI on it using Python 3

mkdir -p cgi-bin/
cargo build
cp target/debug/grease cgi-bin/
python3 -m http.server --cgi 8000
