# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

# nix run .#update commits the newest inputs of the flake and the newest
# crates that the semantic versions and rust-version of Cargo.toml allow,
# each lock file in a commit of its own, and lists the crates held back
{
  perSystem =
    { lib, pkgs, ... }:
    let
      update = pkgs.writeShellApplication {
        name = "update";
        # Nix comes from PATH, so that the caller's own Nix locks the flake
        runtimeInputs = with pkgs; [
          cargo
          git
          gnused
          jq
        ];
        text = ''
          cd "$(git rev-parse --show-toplevel)"
          if ! git diff --quiet HEAD -- flake.lock Cargo.lock; then
            echo 'update: commit or drop the changes to the lock files first' >&2
            exit 1
          fi

          old=$(cat flake.lock)
          if ! log=$(nix flake update 2>&1); then
            printf '%s\n' "$log" >&2
            exit 1
          fi
          if ! git diff --quiet flake.lock; then
            git commit --quiet --message 'flake: update inputs' --message "$(
              jq --raw-output --argjson old "$old" '
                def pin: "\(.rev[:7]) (\(.lastModified | strftime("%Y-%m-%d")))";
                .nodes | to_entries[]
                | select(.value.locked.rev and .value.locked.rev != $old.nodes[.key].locked.rev)
                | if $old.nodes[.key].locked
                  then "\(.key) \($old.nodes[.key].locked | pin) -> \(.value.locked | pin)"
                  else "Adding \(.key) \(.value.locked | pin)"
                  end
              ' flake.lock
            )" flake.lock
          fi

          if ! log=$(cargo update --verbose 2>&1); then
            printf '%s\n' "$log" >&2
            exit 1
          fi
          if ! git diff --quiet Cargo.lock; then
            git commit --quiet --message 'cargo: update dependencies' --message "$(
              sed -En '/^ +Unchanged /d; s/^ +Updating ([^ ]+ v[0-9])/\1/p; s/^ +([A-Z][a-z]+ [^ ]+ v[0-9])/\1/p' <<<"$log"
            )" Cargo.lock
          fi
          sed -En 's/^ +(Unchanged )/\1/p' <<<"$log"
        '';
      };
    in
    {
      apps.update = {
        type = "app";
        program = lib.getExe update;
        meta.description = "Commit the newest inputs and crates that the project allows";
      };
    };
}
