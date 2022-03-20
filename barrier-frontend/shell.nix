{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [ yarn nodejs ];
  buildInputs = with pkgs; [ ];
}
