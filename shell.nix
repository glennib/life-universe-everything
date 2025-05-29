{
  pkgs ? import <nixpkgs> {},
}:
pkgs.mkShell {
  name = "liunev";
  packages = with pkgs; [
  ];
  buildInputs = with pkgs; [
  ];
}
