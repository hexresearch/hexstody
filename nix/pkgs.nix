# To update nix-prefetch-git https://github.com/NixOS/nixpkgs
import ((import <nixpkgs> {}).fetchFromGitHub {
  owner = "NixOS";
  repo = "nixpkgs";
  rev = "21dcccd97d28520d5d22fb545bcdcc72b2cffcd2";
  sha256  = "0h9bwbsvkiqsrkswljnac8jibflfyb1rf3gx867g60qs8nia0n1y";
})
