let
  sources = import ./nix/sources.nix;
  nixpkgs-mozilla = import sources.nixpkgs-mozilla;
  pkgs = import sources.nixpkgs {
    overlays =
      [
        nixpkgs-mozilla
        (self: super:
            let chan = self.rustChannelOf { date = "2022-01-25"; channel = "nightly"; };
            in {
              rustc = chan.rust;
              cargo = chan.rust;
            }
        )
      ];
  };
  naersk = pkgs.callPackage sources.naersk {};
  merged-openssl = pkgs.symlinkJoin { name = "merged-openssl"; paths = [ pkgs.openssl.out pkgs.openssl.dev ]; };
in
naersk.buildPackage {
  name = "hexstody";
  root = pkgs.lib.sourceFilesBySuffices ./. [".rs" ".toml" ".lock" ".html" ".css" ".png" ".sh" ".sql"];
  buildInputs = with pkgs; [ openssl pkgconfig clang llvm llvmPackages.libclang zlib cacert curl postgresql ];
  LIBCLANG_PATH = "${pkgs.llvmPackages.libclang}/lib";
  OPENSSL_DIR = "${merged-openssl}";
  preBuild = ''
    echo "Deploying local PostgreSQL"
    initdb ./pgsql-data --auth=trust
    echo "unix_socket_directories = '$PWD'" >> ./pgsql-data/postgresql.conf
    pg_ctl start -D./pgsql-data -l postgresql.log
    psql --host=$PWD -d postgres -c "create role \"hexstody\" with login password 'hexstody';"
    psql --host=$PWD -d postgres -c "create database \"hexstody\" owner \"hexstody\";"
    cp -r ${./hexstody-db/migrations} ./hexstody-db/migrations
    for f in ./hexstody-db/migrations/*.sql
    do
      echo "Applying $f"
      psql --host=$PWD -U hexstody -d hexstody -f $f
    done
    export DATABASE_URL=postgres://hexstody:hexstody@localhost/hexstody
    echo "Local database accessible by $DATABASE_URL"
  '';
  postInstall = ''
    mkdir -p $out/share/operator
    mkdir -p $out/share/public
    cp -r ${./hexstody-operator/static} $out/share/operator/static
    cp -r ${./hexstody-public/static} $out/share/public/static
    cp -r ${./hexstody-operator/templates} $out/share/operator/templates
    cp -r ${./hexstody-public/templates} $out/share/public/templates
  '';
}
