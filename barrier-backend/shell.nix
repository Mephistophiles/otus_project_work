{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [ gcc ];
  buildInputs = with pkgs; [ openssl openldap pkg-config zlib ];
}
