flake: {
  config,
  lib,
  pkgs,
  ...
}:
with lib; let
  cfg = config.services.timelapse;
  user = "timelapse";
  group = "timelapse";
  initServiceName = "timelapse-init";
  inherit (flake.packages.${pkgs.stdenv.hostPlatform.system}) timelapse;
in {
  options = {
    services = {
      timelapse = {
        enable = mkEnableOption ''
          get pictures from webcam to create a timelapse video
        '';
        instances = mkOption {
          description = "instance definitions";
          default = {};
          type = types.attrsOf (types.submodule {
            options = {
              url = mkOption {
                type = types.str;
                default = "webcam url";
                description = "webcam url";
              };
            };
          });
        };
        interval = mkOption {
          type = types.int;
          default = 10;
          description = "delay between pictures taken";
        };
        dataDir = mkOption {
          type = types.path;
          default = "/var/lib/timelapse";
          description = "directory to store pictures and videos";
        };
        logLevel = mkOption {
          type = types.str;
          default = "warn";
          description = "log level";
        };
      };
    };
  };

  config = lib.mkIf cfg.enable {
    users.users."${user}" = {
      description = "timelapse user";
      isSystemUser = true;
      group = "${group}";
      home = cfg.dataDir;
      createHome = true;
    };

    users.groups.${group} = {};

    systemd.services = let
      x = 1;
      services =
        mapAttrsToList
        (name: instanceCfg: {
          "timelapse-${name}" = {
            description = "webcam timelapse ${name}";

            after = ["${initServiceName}.service" "network-online.target"];
            wants = ["network-online.target"];
            wantedBy = ["multi-user.target"];

            path = with pkgs; [ffmpeg];

            serviceConfig = {
              User = "${user}";
              Group = "${group}";
              Restart = "on-failure";
              ExecStart = ''${lib.getBin timelapse}/bin/timelapse -n ${name} -u ${instanceCfg.url} -i ${toString cfg.interval} -p ${cfg.dataDir}/pics/${name} -s ${cfg.dataDir}/${name}.lock -v ${cfg.dataDir}/videos/${name} -c ${cfg.dataDir}/current'';
              WorkingDirectory = cfg.dataDir;
            };

            environment = {
              RUST_LOG = cfg.logLevel;
            };
          };
        })
        cfg.instances;
    in (mkMerge (services
      ++ [
        {
          "${initServiceName}" = {
            description = "timelapse initialization";
            after = ["network.target"];
            wantedBy = ["multi-user.target"];
            serviceConfig = {
              ExecStart = pkgs.buildPackages.writeShellScript "init-timelapse" ''
                ${pkgs.coreutils}/bin/chown -R ${user}:${group} ${cfg.dataDir}
                ${pkgs.coreutils}/bin/chmod go+rx ${cfg.dataDir}
              '';
              Type = "oneshot";
            };
          };
        }
      ]));
  };
}
