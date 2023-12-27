{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, flake-utils, crane, nixpkgs, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = (import nixpkgs) {
          inherit system overlays;
        };
        
        toolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        });
        src = craneLib.cleanCargoSource (craneLib.path ./.);
        craneLib = crane.lib.${system};

        commonArgs = {
          inherit src;
          strictDeps = true;
          pname = "mage_corp";
          version = "0.1.0";

          buildInputs = [
            # Add additional build inputs here
            pkgs.pkg-config pkgs.clang
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv pkgs.rustPlatform.bindgenHook
            pkgs.darwin.apple_sdk.frameworks.Cocoa
          ];
        };

        devTools = [ toolchain pkgs.lldb pkgs.bacon pkgs.cargo-nextest ];
        
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        mage_corp = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });
      in {
        packages = {
          default = mage_corp;
        };
        devShells.default = pkgs.mkShellNoCC rec {
          nativeBuildInputs = commonArgs.buildInputs ++ devTools;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nativeBuildInputs;

          # LIBCLANG_PATH = "${pkgs.libclang}/lib";
          # shellHook = ''
          #   export DYLD_FALLBACK_LIBRARY_PATH="$(rustc --print sysroot)/lib";
          # '';
        };
      }
    );
}
