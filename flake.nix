{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, flake-utils, naersk, nixpkgs, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = (import nixpkgs) {
          inherit system overlays;
        };
        
        toolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-analyzer" "rust-src" ];
        });
        
        rust_deps = [ toolchain pkgs.lldb pkgs.bacon pkgs.cargo-nextest ];
        bevy_build_deps = with pkgs; [
          pkg-config
          mold clang lld
          makeWrapper
        ];
        bevy_runtime_deps = with pkgs; [
          # udev alsa-lib vulkan-loader pipewire.lib # bevy deps
          # xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr # To use the x11 feature
          # libxkbcommon wayland # To use the wayland feature
          rustPlatform.bindgenHook darwin.apple_sdk.frameworks.Cocoa
        ];
      in {
        devShells.default = pkgs.mkShell rec {
          nativeBuildInputs = bevy_build_deps ++ bevy_runtime_deps ++ rust_deps;
          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath nativeBuildInputs;
          LIBCLANG_PATH = "${pkgs.libclang}/lib";

          # for cargo-nextest, because works using dynamic linking
          shellHook = ''
            export DYLD_FALLBACK_LIBRARY_PATH="$(rustc --print sysroot)/lib";
          '';
        };
      }
    );
}
