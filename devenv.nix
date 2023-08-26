{ pkgs, lib, ... }:

{
  env.RUSTFLAGS = "-C link-arg=-fuse-ld=/run/current-system/sw/bin/mold";
  
  packages = with pkgs; [
    pkg-config udev alsa-lib vulkan-loader
    xorg.libX11 xorg.libXcursor xorg.libXi xorg.libXrandr # To use the x11 feature
    libxkbcommon wayland # To use the wayland feature
  ];
  
  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" ];
  };
  
  
}