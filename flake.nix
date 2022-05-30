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
        packages.kakoune-discord-rc = pkgs.writeTextDir "rc/discord.kak" ''
          ${builtins.readFile ./rc/discord.kak}
          # Set a reference to the kakoune-discord package
          set-option global kakoune_discord_cmd '${packages.kakoune-discord}/bin/kakoune-discord'
        '';
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
