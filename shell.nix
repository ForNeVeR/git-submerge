let
  pkgs = import <nixpkgs> {};
  stdenv = pkgs.stdenv;
in rec {
  gitSubmergeEnv = stdenv.mkDerivation rec {
    name = "git-submerge-env";
    version = "0.0.1";
    src = ./.;
    buildInputs = [
      pkgs.rustUnstable.rustc
      pkgs.rustUnstable.cargo

      pkgs.pkgconfig
      pkgs.openssl
      pkgs.libssh2
      pkgs.libgit2
      pkgs.cmake
    ];
  };
}
