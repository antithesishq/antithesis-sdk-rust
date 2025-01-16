{
  description = "Development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-compat.url = "https://flakehub.com/f/edolstra/flake-compat/1.tar.gz";
    flakelight.url = "github:nix-community/flakelight";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { flakelight, ... } @ inputs: flakelight ./. {
    inherit inputs;
    withOverlays = [
      inputs.rust-overlay.overlays.default
      (final: { inputs', lib, rust-bin, ... }:
        let
          # version:
          # "latest" => latest stable
          # "nightly" => latest nightly
          # "1.61.0" => specific stable version
          craneLib = version: inputs'.crane.lib.overrideToolchain (if version == "nightly" then rust-bin.nightly.latest.default else rust-bin.stable.${version}.default);
          commonArgs = {
            src = ./.;
            pname = "antithesis-sdk-rust-workspace";
            version = "0.0.0";
          };
          workspaceDeps = version: (craneLib version).buildDepsOnly commonArgs;
          workspace = version: (craneLib version).buildPackage (commonArgs // {
            cargoArtifacts = workspaceDeps version;
          });
          workspaceEmptyFeature = version: (craneLib version).buildPackage (commonArgs // {
            cargoArtifacts = workspaceDeps version;
            cargoExtraArgs = "--no-default-features"; # Disable the default `full` feature for builds.
            cargoTestExtraArgs = "-F full"; # But enable the `full` feature when running `cargo test`.
          });
          clippy = version: (craneLib version).cargoClippy (commonArgs // {
            cargoArtifacts = workspaceDeps version;
            cargoClippyExtraArgs = "--all-targets -- -D warnings";
          });
          test = version: (craneLib version).cargoTest (commonArgs // {
            cargoArtifacts = workspaceDeps version;
          });
          doc = version: (craneLib version).cargoDoc (commonArgs // {
            cargoArtifacts = workspaceDeps version;
          });
        in
        {
          inherit craneLib workspaceDeps;
          antithesis-sdk-rust = {
            workspace = workspace "nightly";
            workspaceEmptyFeature = workspaceEmptyFeature "nightly";
            workspaceMSRV = workspace (lib.importTOML ./lib/Cargo.toml).package.rust-version;
            clippy = clippy "nightly";
            test = test "nightly";
            doc = doc "nightly";
          };
        })
    ];

    packages = rec {
      default = workspace;
      workspace = { antithesis-sdk-rust }: antithesis-sdk-rust.workspace;
      doc = { antithesis-sdk-rust }: antithesis-sdk-rust.doc;
    };

    apps = rec {
      default = simple;
      simple = pkgs: "${pkgs.antithesis-sdk-rust-workspace}/bin/simple";
    };

    devShells.default = pkgs: {
      inputsFrom = with pkgs; [ antithesis-sdk-rust.workspace ];
      packages = with pkgs; [ rust-analyzer cargo-msrv ];
    };

    # TODO: Perform semver check.
    checks = { antithesis-sdk-rust, ... }: {
      inherit (antithesis-sdk-rust) workspaceMSRV workspaceEmptyFeature clippy test;
    };

    # TODO: Decide whether we want auto formatting.
    # formatters = pkgs: {
    #   "*.rs" = "${pkgs.rustfmt}/bin/rustfmt";
    #   "*.nix" = "${pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt";
    # };

    flakelight.builtinFormatters = false;
  };
}
