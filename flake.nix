{
  # Tremendous thanks to @oati for her help
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, flake-utils }: 
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        python-package-list = p: with p; [
          pip
          pandas
          numpy
          certifi
          colorama
          pysocks
          requests
          requests-futures
          stem
          torrequest
          openpyxl
          exrex
        ];
        python = pkgs.python311.withPackages python-package-list;
      in
      {
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ openssl.dev pkg-config ];
          buildInputs = with pkgs; [ cargo rustc rustfmt rust-analyzer clippy python ];
          shellHook = 
            ''
            '';
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        };
    });
}
