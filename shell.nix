{
  pkgs ? import <nixpkgs> {},
}:

with pkgs;
mkShell rec {
  nativeBuildInputs = [
    pkg-config
  ];
  name = "liunev";
  packages = with pkgs; [
  ];
  buildInputs = with pkgs; [
    xorg.libX11
	xorg.libXcursor
	xorg.libXi
	xorg.libXrandr
	libGL
	libxkbcommon
  ];
  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;
}
