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
      in
      {
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ openssl.dev pkg-config ];
          buildInputs = with pkgs; [ cargo rustc rustfmt rust-analyzer clippy  ];
          shellHook = 
            ''
            '';
          RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
        };
    });
}
