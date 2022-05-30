{
  inputs = {
    nixpkgs.url = github:nixOS/nixpkgs/nixpkgs-unstable;
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, naersk }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages."${system}";
        naersk-lib = naersk.lib."${system}";
      in
      rec {
        # `nix build`
        packages.kakoune-discord = naersk-lib.buildPackage {
          pname = "kakoune-discord";
          root = ./.;
        };
        defaultPackage = packages.kakoune-discord;

        # `nix run`
        apps.kakoune-discord = flake-utils.lib.mkApp {
          drv = packages.kakoune-discord;
        };
        defaultApp = apps.kakoune-discord;

        # `nix develop`
        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [ rustc cargo rust-analyzer libiconv ];
        };
      }
    );
}
