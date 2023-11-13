{ pkgs, lib, config, ... }:

with lib;

let
  cfg = config.programs.fsh;
in
{
  options.programs.fsh = {
    enable = mkOption {
      type = types.bool;
      default = false;
      description = ''
      Enables the fsh prompt for the fish shell.
      programs.fish.enable must also be set to true for this option to have effect.
      '';
    };
  };

  config = mkIf cfg.enable {
    home.packages = [ pkgs.fsh ];
    programs.fish.functions.fish_prompt.body = ''
    set FSH_LAST_STATUS $status
    if test "$SSH_CLIENT" != "" || test "$SSH_TTY" != "" | test "$SSH_CONNECTION" != ""
      set --erase FSH_NO_HOSTNAME
    else
      set -x FSH_NO_HOSTNAME 1
    end
    fsh $FSH_LAST_STATUS
    '';
  };
}
