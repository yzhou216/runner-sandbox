# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: GPL-3.0-or-later

{
  perSystem =
    {
      config,
      lib,
      pkgs,
      system,
      crate,
      sourceDateEpoch,
      darwinBuild,
      ...
    }:
    let
      # glibc of Nixpkgs, which cinq links against, so that other
      # distributions need this one or a newer one
      glibc = lib.versions.majorMinor pkgs.glibc.version;
      # cinq for other distributions of GNU/Linux, with their loader and
      # libraries in place of the Nix store's
      portable =
        let
          directory = if pkgs.stdenv.hostPlatform.isx86_64 then "lib64" else "lib";
          loader = "/${directory}/${baseNameOf pkgs.stdenv.cc.bintools.dynamicLinker}";
        in
        pkgs.runCommand "${crate.name}-portable" { nativeBuildInputs = [ pkgs.patchelf ]; } ''
          mkdir -p "$out/usr"
          cp -r ${config.packages.cinq}/{bin,share} "$out/usr"
          chmod -R u+w "$out/usr"
          patchelf --set-interpreter ${loader} --remove-rpath "$out/usr/bin/cinq"
          cp -r --no-preserve=mode ${config.packages.doc}/share "$out/usr"
        '';
      # Package that depends on the unwinder of GCC, which Rust's standard
      # library takes, but on no library of the GPU, without which cinq
      # draws in software
      nfpm =
        packager:
        pkgs.runCommand "${crate.name}-${packager}"
          {
            nativeBuildInputs = [ pkgs.nfpm ];
            env.SOURCE_DATE_EPOCH = sourceDateEpoch;
            nfpmConfig = builtins.toJSON rec {
              inherit (crate)
                name
                version
                description
                license
                ;
              arch = pkgs.stdenv.hostPlatform.go.GOARCH;
              maintainer = "Yiyu Zhou <yiyu@yiyuzhou.io>";
              section = "sound";
              archlinux.packager = maintainer;
              # Files alone, leaving their directories to the system
              contents = [
                {
                  src = "usr/";
                  dst = "/usr";
                }
              ];
              overrides = {
                deb.depends = [
                  "libc6 (>= ${glibc})"
                  "libasound2t64 | libasound2"
                  "libgcc-s1"
                  "libdbus-1-3"
                  "libwayland-client0"
                  "libxkbcommon0"
                ];
                rpm.depends = [
                  "glibc >= ${glibc}"
                  "alsa-lib"
                  "libgcc"
                  "dbus-libs"
                  "libwayland-client"
                  "libxkbcommon"
                ];
                archlinux.depends = [
                  "glibc"
                  "alsa-lib"
                  "gcc-libs"
                  "dbus"
                  "wayland"
                  "libxkbcommon"
                ];
              };
            };
            passAsFile = [ "nfpmConfig" ];
          }
          ''
            cp -r ${portable}/usr usr
            chmod -R u+w usr
            mkdir "$out"
            nfpm package --config "$nfpmConfigPath" --packager ${packager} --target "$out"
          '';
      # mkbom of a release that writes past a buffer for every file but
      # links, which the hardening of Nixpkgs stops, and that never ends its
      # loop over the options where char is unsigned, as on aarch64-linux,
      # until Nixpkgs takes hogliux/bomutils#34 and the fix before it
      bomutils = pkgs.bomutils.overrideAttrs (old: {
        patches = old.patches or [ ] ++ [
          (pkgs.fetchpatch {
            url = "https://github.com/hogliux/bomutils/commit/14f5d09d6c62fef7539ebdd23ebcd42ab54f7351.patch";
            hash = "sha256-oPxAJTjD6j7aQ7Oko35s9VtCFOLrK/P29xR9XHLETy0=";
          })
          (pkgs.fetchpatch {
            url = "https://github.com/hogliux/bomutils/commit/b05cac6aa2c819c4103c6415ace975d5074e5352.patch";
            hash = "sha256-tWNLcHJW7rAS9kjCU3RcDedEogFtmlYRDEzMISYv0XQ=";
          })
        ];
      });
      # Installer package for Darwin, unsigned, with the app in
      # /Applications and the rest in /usr/local, a component each, so that
      # no payload holds / or /usr.  The root of each payload takes the mode
      # and owner of its directory on Darwin, and the package keeps the
      # permissions of directories that are already there.
      darwinPkg =
        pkgs.runCommand "${crate.name}-darwin-pkg"
          {
            nativeBuildInputs = [
              bomutils
            ]
            ++ (with pkgs; [
              cpio
              libfaketime
              xar
            ]);
            env = {
              SOURCE_DATE_EPOCH = sourceDateEpoch;
              # Zone of the local time that xar writes as the creation time,
              # which a build on Darwin takes from the machine otherwise
              TZ = "UTC";
            };
          }
          ''
            # component <name> <identifier> <install location> <uid> <gid>
            component() {
              find "$1" -exec touch -h -d "@$SOURCE_DATE_EPOCH" {} +
              mkdir -p "flat/$1.pkg"
              (cd "$1" && find . | LC_ALL=C sort | cpio -o --quiet --format odc --reproducible --owner "$4:$5") |
                gzip -9n > "flat/$1.pkg/Payload"
              mkbom -u "$4" -g "$5" "$1" "flat/$1.pkg/Bom"
              # Sizes of files alone, since those of directories differ
              # between file systems
              kbytes=$(find "$1" -type f -printf '%s\n' | awk '{ s += $1 } END { print int((s + 1023) / 1024) }')
              cat > "flat/$1.pkg/PackageInfo" <<EOF
            <?xml version="1.0" encoding="utf-8"?>
            <pkg-info format-version="2" identifier="$2" version="${crate.version}" install-location="$3" auth="root" overwrite-permissions="false">
              <payload numberOfFiles="$(find "$1" | wc -l)" installKBytes="$kbytes"/>
            </pkg-info>
            EOF
              refs+="  <pkg-ref id=\"$2\" version=\"${crate.version}\" installKBytes=\"$kbytes\">#$1.pkg</pkg-ref>"$'\n'
            }

            mkdir -p app local/bin
            cp -r ${darwinBuild}/Applications/cinq.app app
            cp -r ${darwinBuild}/share local
            chmod -R u+w app local
            chmod 775 app
            cp -r --no-preserve=mode ${config.packages.doc}/share/doc local/share
            # Files of freedesktop.org, which Darwin has no use for
            rm -r local/share/{applications,icons,mime}
            ln -s /Applications/cinq.app/Contents/MacOS/cinq local/bin
            component app io.yiyuzhou.cinq /Applications 0 80
            component local io.yiyuzhou.cinq.local /usr/local 0 0

            cat > flat/Distribution <<EOF
            <?xml version="1.0" encoding="utf-8"?>
            <installer-gui-script minSpecVersion="1">
              <title>cinq</title>
              <options customize="never" require-scripts="false" hostArchitectures="arm64"/>
              <choices-outline>
                <line choice="default"/>
              </choices-outline>
              <choice id="default">
                <pkg-ref id="io.yiyuzhou.cinq"/>
                <pkg-ref id="io.yiyuzhou.cinq.local"/>
              </choice>
            $refs</installer-gui-script>
            EOF

            mkdir "$out"
            cd flat
            # Creation time of the archive from the clock, which xar takes
            # from nothing else, and a buffer of a fixed size, since xar
            # compresses its table of contents in pieces of the size that
            # the file system prefers otherwise
            faketime -f "$(date -u -d "@$SOURCE_DATE_EPOCH" '+%F %T')" \
              xar --distribution --compression none --rsize 65536 \
              -cf "$out/${crate.name}-${crate.version}-aarch64-darwin.pkg" \
              Distribution {app,local}.pkg/{Bom,PackageInfo,Payload}
          '';
    in
    {
      packages =
        lib.optionalAttrs pkgs.stdenvNoCC.hostPlatform.isLinux (
          lib.genAttrs [ "deb" "rpm" "archlinux" ] nfpm
        )
        // lib.optionalAttrs (pkgs.stdenvNoCC.hostPlatform.isLinux || system == "aarch64-darwin") {
          darwin-pkg = darwinPkg;
        };
    };
}
