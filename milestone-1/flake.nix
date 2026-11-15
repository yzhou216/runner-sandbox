{
  description = "Build and run Bevy projects";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell (finalAttrs: {
          packages = with pkgs; [ cargo-nextest ];

          nativeBuildInputs = with pkgs; [ pkg-config ];

          buildInputs = with pkgs; [
            udev
            alsa-lib-with-plugins
            vulkan-loader

            # X11 support
            libx11
            libxcursor
            libxi
            libxrandr

            # Wayland support
            libxkbcommon
            wayland
          ];

          env.LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath finalAttrs.buildInputs;
        });
      }
    );
}
