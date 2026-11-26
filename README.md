<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# geoai's object storage, in NixOS virtual machines

Tests the object storage of geoai on ubuntu-latest, in NixOS virtual machines
under KVM.  `geoai/` is a snapshot of the working tree of
[yiyu/geoai](https://codeberg.org/yiyu/geoai) at commit 01d1974 of the
`man-pages` branch, plus the uncommitted changes that let geoai read and write
rasters by URL (`src/storage.rs`, `doc/storage.md`, and the `format` and CLI
changes they need).  `.gitattributes` is left out, so that the test data is
stored as plain files rather than through Git LFS.  geoai ignores its
`flake.lock`, so its flake resolved nixpkgs-unstable at the time of the run:
619c39f, dated 2026-11-22.

`storage.nix` defines two machines:

- `storage` runs Garage 2.5.0 behind nginx, which serves Garage's S3 API on
  port 443 and a directory holding `PGA_250y.tif` on port 8443, both over
  TLS with a certificate from a CA generated at build time.  Only those two
  ports are open.
- `client` has geoai, s5cmd, and GDAL, trusts the test CA through
  `security.pki.certificateFiles`, and has flakes enabled.  geoai runs with
  `SSL_CERT_FILE` and `NIX_SSL_CERT_FILE` unset, so it finds the CA in the
  system trust store, `/etc/ssl/certs/ca-certificates.crt`.

The job `flake-check` runs `nix flake check` on the snapshot.

## Result

Run 45789909893 (commit f182eb0): both jobs green.  The test script took 58
seconds, and the job `vm` took 5 to 6 minutes, most of it spent building
geoai.  `flake-check` passed `cargo-clippy`, `cargo-test`, `cargo-fmt`,
`rumdl`, and `reuse`.

| Check | Outcome |
| --- | --- |
| Earthquake output written to `s3://`, local input | identical to the output written to disk (`cmp`) |
| `pga` read from `s3://` | identical output |
| `pga` read from `https://storage:8443/` | identical output |
| `--like` an `s3://` object, the grid of `hazard_earthquake.tif` | identical to the local `--like` output, which differs from the output on the grid of `pga` |
| 55 MB float output, hazard `scaled` | identical; nginx logged several `PUT ...?partNumber=` and one `POST ...?uploadId=`, all 200 |
| `geoai info s3://...` | same text as for the local file |
| `gdalinfo /vsis3/...` over TLS | size 9825 x 7964, band description `scaled`; nginx logged a 206 response, a range read |
| Missing object | fails: `Object at location runs/missing.tif not found: ... 404 Not Found` |
| Wrong secret key | fails: `The operation lacked the necessary privileges ... 403 Forbidden` |
| `SSL_CERT_FILE` naming the Mozilla bundle, without the test CA | fails: `invalid peer certificate: UnknownIssuer` |

## Notes

- `PGA_250y.tif` was read from the store five times in three runs of geoai:
  twice in each of the two runs where it gives the output its grid, and
  once in the run with `--like`.  That is the double download that
  `doc/storage.md` describes.
- The certificate error was retried ten times, in 2 seconds, before geoai
  failed: `object_store` retries a failed connection whatever its cause.
- Run 45785394563 failed in `flake-check` alone: the rumdl of nixpkgs 619c39f
  enables MD076, which requires blank lines between the items of the
  numbered list in `doc/storage.md`, one of whose items holds a code block.
  The second commit adds them, as does the working tree of geoai.
