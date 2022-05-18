{ config, pkgs, lib, ... }:
with lib;  # use the functions from lib, such as mkIf
let
  # the values of the options set for the service by the user of the service
  cfg = config.services.hexstody-btc;
in {
  ##### interface. here we define the options that users of our service can specify
  options = {
    # the options for our service will be located under services.hexstody-btc
    services.hexstody-btc = {
      enable = mkOption {
        type = types.bool;
        default = false;
        description = ''
          Whether to enable hexstody BTC adapter service by default.
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

      btcNode = mkOption {
        type = types.str;
        default = "127.0.0.1:8332/wallet/hexstody";
        description = ''
          Host and port where BTC RPC node is located.
        '';
      };
      rpcUser = mkOption {
        type = types.str;
        default = "bitcoin";
        description = ''
          Which name of bitcoin RPC user to use.
        '';
      };
      passwordFile = mkOption {
        type = types.str;
        default = "/run/keys/hexstodybtcrpc";
        description = ''
          Location of file with password for RPC.
        '';
      };
      passwordFileService = mkOption {
        type = types.str;
        default = "hexstodybtcrpc-key.service";
        description = ''
          Service that indicates that passwordFile is ready.
        '';
      };
      secretKey = mkOption {
        type = types.str;
        default = "/run/keys/hexstodybtccookieskey";
        description = ''
          Location of file with cookies secret key.
        '';
      };
      secretKeyService = mkOption {
        type = types.str;
        default = "hexstodybtccookies-key.service";
        description = ''
          Service that indicates that secretKey is ready.
        '';
      };
    };
  };

  ##### implementation
  config = mkIf cfg.enable { # only apply the following settings if enabled
    # User to run the node
    users.users.hexstody-btc = {
      name = "hexstody-btc";
      group = "hexstody-btc";
      description = "hexstody-btc daemon user";
      isSystemUser = true;
    };
    users.groups.hexstody-btc = {};
    # Create systemd service
    systemd.services.hexstody-btc = {
      enable = true;
      description = "Hexstody BTC adapter";
      after = ["network.target" cfg.passwordFileService cfg.secretKeyService];
      wants = ["network.target" cfg.passwordFileService cfg.secretKeyService];
      script = ''
        export HEXSTODY_BTC_NODE_PASSWORD=$(cat ${cfg.passwordFile} | xargs echo -n)
        export HEXSTODY_BTC_SECRET_KEY=$(cat ${cfg.secretKey} | xargs echo -n)
        ${cfg.package}/bin/hexstody-btc serve \
            --address ${cfg.host} \
            --node-url ${cfg.btcNode} \
            --node-user ${cfg.rpcUser} \
            --port ${builtins.toString cfg.port} 
      '';
      serviceConfig = {
          Restart = "always";
          RestartSec = 30;
          User = "hexstody-btc";
          LimitNOFILE = 65536;
        };
      wantedBy = ["multi-user.target"];
    };
  };
}
