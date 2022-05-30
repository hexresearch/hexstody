{ containerTag ? "latest"
, prefixName ? ""
}:
let
  sources = import ./sources.nix;
  pkgs = import sources.nixpkgs {};
  hexstody = import ../default.nix;

  baseImage = pkgs.dockerTools.pullImage {
      imageName = "debian";
      imageDigest = "sha256:7d8264bf731fec57d807d1918bec0a16550f52a9766f0034b40f55c5b7dc3712";
      sha256 = "sha256-PwMVlEk81ALRwDCSdb9LLdJ1zr6tn4EMxcqtlvxihnE=";
    };

  # As we place all executables in single derivation the derivation takes them
  # from it and allows us to make thin containers for each one.
  takeOnly = name: path: pkgs.runCommandNoCC "only-${name}" {} ''
    mkdir -p $out
    cp ${path} $out/${name}
  '';
  takeFolder = name: path: innerPath: pkgs.runCommandNoCC "folder-${name}" {} ''
    mkdir -p $out/${innerPath}
    cp -r ${path}/* $out/${innerPath}
  '';

  mkDockerImage = name: cnts: pkgs.dockerTools.buildImage {
    name = "${prefixName}${name}";
    fromImage = baseImage;
    tag = containerTag;
    contents = cnts;
  };

  hexstody-container = mkDockerImage "hexstody" [
    (takeOnly "hexstody-hot" "${hexstody}/bin/hexstody-hot")
    (takeOnly "hexstody-btc" "${hexstody}/bin/hexstody-btc")
    (takeOnly "operator-keygen" "${hexstody}/bin/operator-keygen")
    (takeOnly "wait-for-it.sh" "${hexstody.src}/docker/wait-for-it.sh")
    (takeFolder "operator-static" "${hexstody}/share/operator/static" "/operator/static")
    (takeFolder "operator-templates" "${hexstody}/share/operator/templates" "/operator/templates")
    (takeFolder "public-static" "${hexstody}/share/public/static" "/public/static")
    (takeFolder "public-templates" "${hexstody}/share/public/templates" "/public/templates")
  ];
in { inherit
  hexstody-container
  ;
}
