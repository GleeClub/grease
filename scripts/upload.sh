#!/bin/bash

# Build binaries and copy them to the school's server
# Make sure you have SSH access set up for the below address

cargo build --release --target=x86_64-unknown-linux-musl
for bin in "grease admin_tools send_emails"; do
    scp "target/release/$bin" mensgleeclub@mensgleeclub.gatech.edu:/cgi-bin/
done
