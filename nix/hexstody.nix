{config, lib, pkgs, ...}:
{
    imports = [
        ./hexstody-btc.nix
        ./hexstody-hot.nix
    ];
    config = {
        nixpkgs.overlays = [
            (import ./overlay.nix)
        ];
        services.hexstody-btc = {
            enable = true;
        };
        services.bitcoin.extraConfig = ''
            wallet=hexstody
        '';
        services.hexstody-hot = {
            enable = true;
        };
        systemd.services = {
            hexstodybtcrpc-key = {
                enable = true;
                description = "BTC key is provided";
                wantedBy = [ "network.target" ];
                serviceConfig.Type = "oneshot";
                serviceConfig.RemainAfterExit = true;
                script =
                ''
                    echo "BTC key is done"
                '';
            };
            hexstodydb-key = {
                enable = true;
                description = "Database password is provided";
                wantedBy = [ "network.target" ];
                serviceConfig.Type = "oneshot";
                serviceConfig.RemainAfterExit = true;
                script =
                ''
                    echo "Database password is done"
                '';
            };
            hexstodybtccookies-key = {
                enable = true;
                description = "Cookies key is provided";
                wantedBy = [ "network.target" ];
                serviceConfig.Type = "oneshot";
                serviceConfig.RemainAfterExit = true;
                script =
                ''
                    echo "Cookies key is done"
                '';
            };
            hexstodycookies-key = {
                enable = true;
                description = "Cookies key is provided";
                wantedBy = [ "network.target" ];
                serviceConfig.Type = "oneshot";
                serviceConfig.RemainAfterExit = true;
                script =
                ''
                    echo "Cookies key is done"
                '';
            };
        };
        services.postgresql = {
            ensureDatabases = [ "hexstody" ];
            ensureUsers = [
                { 
                    name = "hexstody";
                    ensurePermissions."DATABASE hexstody" = "ALL PRIVILEGES";
                }
            ];
        };
    };
}