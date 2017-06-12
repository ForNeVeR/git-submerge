let
  pkgs = import <nixpkgs> {};
  stdenv = pkgs.stdenv;
  funs = pkgs.callPackage ./rust-nightly.nix {};

  # Archive is here: https://static.rust-lang.org/dist/index.html
  rustNightly = funs.rust {
    date = "2017-06-12";
    hash = "1jdi078hk56hi2jb39qivswh5lc1a53q41k9pgmxmb9yfvi8v2x2";
  };

in rec {
  gitSubmergeEnv = stdenv.mkDerivation rec {
    name = "git-submerge-env";
    version = "0.0.1";
    src = ./.;
    buildInputs = [
      rustNightly

      pkgs.pkgconfig
      pkgs.openssl
      pkgs.libssh2
      pkgs.libgit2
      pkgs.cmake
      pkgs.zlib
    ];
  };
}
