#!/bin/bash

cargo build --release --target=x86_64-unknown-linux-musl && \
    curl "https://gleeclub.gatech.edu/cgi-bin/admin_tools/upload_api" \
    -H "token: $GREASE_TOKEN" -H "Content-Type: application/zip" \
    --data-binary "@target/x86_64-unknown-linux-musl/release/grease_api"
