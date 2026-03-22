self: {
  config,
  lib,
  pkgs,
  ...
}: let
  cfg = config.services.short-signed-id;
in {
  options.services.short-signed-id = {
    instances = lib.mkOption {
      default = {};
      type = lib.types.attrsOf (lib.types.submodule ({name, ...}: {
        options = {
          bind = lib.mkOption {
            type = lib.types.str;
          };
          keyFile = lib.mkOption {
            type = lib.types.str;
          };
          user = lib.mkOption {
            type = lib.types.str;
          };
          group = lib.mkOption {
            type = lib.types.str;
          };
        };
      }));
    };
  };

  config = {
    systemd.services = lib.mapAttrs' (name: instance:
      lib.nameValuePair "short-signed-id-${name}" {
        wantedBy = ["multi-user.target"];
        serviceConfig = {
          ExecStart = "${self.packages.${pkgs.stdenv.hostPlatform.system}.default}/bin/short_signed_id --bind ${instance.bind} --key-file ${instance.keyFile}";
          Restart = "always";
          RestartSec = 3;
          User = instance.user;
          Group = instance.group;
        };
      })
    cfg.instances;

    users = let
      concatMapAttrsRecursive = f: set:
        lib.foldl'
        (acc: name: lib.recursiveUpdate acc (f name set.${name}))
        {}
        (lib.attrNames set);
    in
      concatMapAttrsRecursive (name: instance: {
        groups.${instance.group} = {};
        users.${instance.user} = {
          group = instance.group;
          isSystemUser = true;
        };
      })
      cfg.instances;
  };
}
