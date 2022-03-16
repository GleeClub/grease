Grease API
---------

The new and improved JSON backend for the Georgia Tech Glee Club's official website.

To view the layout, please check out the `wiki`.

This backend is written in [Rust].

### Building:

To work on this project, you'll need to have [Rust] Nightly installed. Once you install
rust on the `nightly` toolchain, install the `Musl` compilation target with the following
command:

```bash
rustup target add x86_64-unknown-linux-musl
```

Once you have that installed, you can use the [simple build script](./build_and_upload.sh)
to build the API with full optimization and upload it to the web hosting platform.
(You'll need to be on Georgia Tech's network or the official GT VPN to upload over scp
for now.)


[Rust]: https://www.rust-lang.org/learn/get-started
