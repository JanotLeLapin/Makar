{ rust-bin
, rust-analyzer
, mkShell
}: mkShell {
  packages = [
    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
    rust-analyzer
  ];
}
