{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = [
    pkgs.just
    pkgs.rustup
    pkgs.cargo-insta
  ];

  shellHook = ''
    export RUST_BACKTRACE=1
  '';
}
