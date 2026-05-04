{
  description = "Development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";
    flakelight.url = "github:nix-community/flakelight";
    crane.url = "github:ipetkov/crane";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { flakelight, ... } @ inputs: flakelight ./. {
    inherit inputs;
    withOverlays = [
      inputs.rust-overlay.overlays.default
      (final: { rust-bin, ... }:
        let
          craneLib = (inputs.crane.mkLib final).overrideToolchain rust-bin.nightly.latest.default;
          commonArgs = {
            src = ./lib;
            pname = "antithesis-sdk-rust-workspace";
            version = "0.0.0";
          };
          workspaceDeps = craneLib.buildDepsOnly commonArgs;
          workspace = craneLib.buildPackage (commonArgs // {
            cargoArtifacts = workspaceDeps;
          });
          workspaceEmptyFeature = craneLib.buildPackage (commonArgs // {
            cargoArtifacts = workspaceDeps;
            cargoExtraArgs = "--no-default-features"; # Disable the default `full` feature for builds.
            cargoTestExtraArgs = "-F full -F rand_v0_8"; # But enable the `full` and `rand_v0_8` feature when running `cargo test`.
          });
          clippy = craneLib.cargoClippy (commonArgs // {
            cargoArtifacts = workspaceDeps;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
          test = craneLib.cargoTest (commonArgs // {
            cargoArtifacts = workspaceDeps;
          });
          doc = craneLib.cargoDoc (commonArgs // {
            cargoArtifacts = workspaceDeps;
          });
        in
        {
          antithesis-sdk-rust = {
            inherit workspace workspaceEmptyFeature clippy test doc;
          };
        })
    ];

    packages = rec {
      default = workspace;
      workspace = pkgs: pkgs.antithesis-sdk-rust.workspace;
      doc = pkgs: pkgs.antithesis-sdk-rust.doc;
    };

    apps = rec {
      default = simple;
      simple = pkgs: "${pkgs.antithesis-sdk-rust-workspace}/bin/simple";
    };

    devShells.default = pkgs: {
      inputsFrom = with pkgs; [ antithesis-sdk-rust.workspace ];
      packages = with pkgs; [ rust-analyzer cargo-msrv cargo-semver-checks ];
    };

    # TODO: Perform semver check.
    checks = { antithesis-sdk-rust, ... }: {
      inherit (antithesis-sdk-rust) workspaceEmptyFeature clippy test;
    };

    # TODO: Decide whether we want auto formatting.
    # formatters = pkgs: {
    #   "*.rs" = "${pkgs.rustfmt}/bin/rustfmt";
    #   "*.nix" = "${pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt";
    # };

    flakelight.builtinFormatters = false;
  };
}
