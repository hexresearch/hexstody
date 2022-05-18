{ config, pkgs, lib, ... }:
with lib;  # use the functions from lib, such as mkIf
let
  # the values of the options set for the service by the user of the service
  cfg = config.services.hexstody-hot;
in {
  ##### interface. here we define the options that users of our service can specify
  options = {
    # the options for our service will be located under services.hexstody-hot
    services.hexstody-hot = {
      enable = mkOption {
        type = types.bool;
        default = false;
        description = ''
          Whether to enable hexstody hot wallet service by default.
        '';
      };
      package = mkOption {
        type = types.package;
        default = pkgs.hexstody;
        description = ''
          Which package to use with the service.
        '';
      };
      port = mkOption {
        type = types.int;
        default = 8180;
        description = ''
          Which port the BTC adapter listen to serve API.
        '';
      };
      host = mkOption {
        type = types.str;
        default = "0.0.0.0";
        description = ''
          Which hostname is binded to the node.
        '';
      };

      btcModule = mkOption {
        type = types.str;
        default = "http://127.0.0.1:8180";
        description = ''
          Host and port where BTC adapter service is located.
        '';
      };
      databaseHost = mkOption {
        type = types.str;
        default = "localhost:5432";
        description = ''
          Connection host to the database.
        '';
      };
      databaseName = mkOption {
        type = types.str;
        default = "hexstody";
        description = ''
          Database name.
        '';
      };
      databaseUser = mkOption {
        type = types.str;
        default = "hexstody";
        description = ''
          User name for database.
        '';
      };
      passwordFile = mkOption {
        type = types.str;
        default = "/run/keys/hexstodydb";
        description = ''
          Location of file with password for database.
        '';
      };
      passwordFileService = mkOption {
        type = types.str;
        default = "hexstodydb-key.service";
        description = ''
          Service that indicates that passwordFile is ready.
        '';
      };
      secretKey = mkOption {
        type = types.str;
        default = "/run/keys/hexstodycookieskey";
        description = ''
          Location of file with cookies secret key.
        '';
      };
      secretKeyService = mkOption {
        type = types.str;
        default = "hexstodycookies-key.service";
        description = ''
          Service that indicates that secretKey is ready.
        '';
      };
    };
  };

  ##### implementation
  config = mkIf cfg.enable { # only apply the following settings if enabled
    # User to run the node
    users.users.hexstody = {
      name = "hexstody";
      group = "hexstody";
      extraGroups = [ "tor" ];
      description = "hexstody daemon user";
      isSystemUser = true;
    };
    users.groups.hexstody = {};
    # Create systemd service
    systemd.services.hexstody-hot = {
      enable = true;
      description = "Hexstody hot wallet";
      after = ["network.target" cfg.passwordFileService cfg.secretKeyService];
      wants = ["network.target" cfg.passwordFileService cfg.secretKeyService];
      script = ''
        export DB_PASSWORD=$(cat ${cfg.passwordFile} | xargs echo -n)
        export DATABASE_URL="postgresql://${cfg.databaseUser}:$DB_PASSWORD@${cfg.databaseHost}/${cfg.databaseName}"
        export HEXSTODY_SECRET_KEY=$(cat ${cfg.secretKey} | xargs echo -n)
        cd ${cfg.package}/share
        ${cfg.package}/bin/hexstody-hot \
            --btc-module ${cfg.btcModule} \
            --static-path ${cfg.package}/share/static \
            serve
      '';
      serviceConfig = {
          Restart = "always";
          RestartSec = 30;
          User = "hexstody";
          LimitNOFILE = 65536;
        };
      wantedBy = ["multi-user.target"];
    };
  };
}
