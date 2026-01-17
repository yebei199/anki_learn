{
  description = "Rust musl build environment";

  inputs = {
    nixpkgs.url = "nixpkgs";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkgsCross.musl64.stdenv.cc
            pkg-config
            sccache
            cargo-cross
            clang
            cmake
          ];

          shellHook = ''
            export CC_x86_64_unknown_linux_musl=x86_64-unknown-linux-musl-gcc
            export CXX_x86_64_unknown_linux_musl=x86_64-unknown-linux-musl-g++
            export AR_x86_64_unknown_linux_musl=x86_64-unknown-linux-musl-ar
            
            # Create a temporary bin directory for aliases
            mkdir -p .nix-bins
            ln -sf $(which x86_64-unknown-linux-musl-gcc) .nix-bins/x86_64-linux-musl-gcc
            ln -sf $(which x86_64-unknown-linux-musl-g++) .nix-bins/x86_64-linux-musl-g++
            ln -sf $(which x86_64-unknown-linux-musl-ar) .nix-bins/x86_64-linux-musl-ar
            export PATH=$PWD/.nix-bins:$PATH

            # Ensure sccache is configured to be used
            export RUSTC_WRAPPER=${pkgs.sccache}/bin/sccache
          '';
        };
      }
    );
}
