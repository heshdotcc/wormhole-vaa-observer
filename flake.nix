{
  description = "Wormhole VAA Observer Development Shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
            protobuf
            just
            cargo-watch
          ];

          buildInputs = with pkgs; [
            rustc
            cargo
            rust-analyzer
            openssl
            nodejs_22
            deno
            kubectl
            helm
            docker
            zsh 
            grml-zsh-config
          ];

          shellHook = ''
            export PROTOC="${pkgs.protobuf}/bin/protoc"
            export PROTOC_INCLUDE="${pkgs.protobuf}/include"            
            export SHELL=$(which zsh)
            echo "🌀 Welcome to the wormhole-vaa-observer development shell!"
            if [ -t 1 ]; then
              exec zsh
            fi
          '';
        };
      }
    );
}
