#!/bin/bash

# Build `grease`, put it in the `cgi-bin/` folder,
# and run CGI on it using Python 3

cargo build --release && \
    mkdir -p cgi-bin/ && \
    cp target/release/grease cgi-bin/ && \
    python3 -m http.server --cgi 8000
