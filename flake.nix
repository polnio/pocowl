{
  description = "My own wayland compositor in rust";
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  outputs =
    { self, nixpkgs }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          lib = pkgs.lib;
        in
        {
          default = pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              cargo
              rustc
              # GLFW
              cmake
              pkg-config
              wayland
              libxkbcommon
              libffi
              libx11
              libxrandr
              libxinerama
              libxcursor
              libxi
            ];
            LD_LIBRARY_PATH = lib.makeLibraryPath (
              with pkgs;
              [
                libGL
                libxkbcommon
              ]
            );
          };
        }
      );
    };
}
