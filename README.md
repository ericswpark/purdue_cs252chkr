# cs252chkr

Retrieves and displays useful metadata about the repository for grading purposes

## Building

- Install `rustup`
- Install the Rust compiler and toolchain with `rustup`
- Build a release executable using the following command:

```
cargo build --release
```

### Troubleshooting

If you get an error about building the `openssl-sys` crate, you may need to install `openssl` and the `pkg-config` library on Linux. Windows should build without depending on system libraries.

If you are on NixOS, there is a `flake.nix` with the associated `direnv` file that allows for building within a `devshell`. Run `direnv allow` in order to automatically enter the `devshell` upon entering the project directory.