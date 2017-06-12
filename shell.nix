let
  pkgs = import <nixpkgs> {};
  stdenv = pkgs.stdenv;
in rec {
  gitSubmergeEnv = stdenv.mkDerivation rec {
    name = "git-submerge-env";
    version = "0.0.1";
    src = ./.;
    buildInputs = with pkgs.rustUnstable; [
      rustc
      cargo
    ];
  };
}
