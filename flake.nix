{
  description = "Development environment for the R47 OFF-image generator";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { nixpkgs, ... }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };
      guiLibraries = with pkgs; [
        libGL
        libxkbcommon
        wayland
        libX11
        libXcursor
        libXi
        libXrandr
      ];
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          cargo
          rustc
          rustfmt
          clippy
          rust-analyzer
          pkg-config
        ];

        buildInputs = guiLibraries;
        RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";

        shellHook = ''
          export LD_LIBRARY_PATH="${pkgs.lib.makeLibraryPath guiLibraries}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
        '';
      };
    };
}
