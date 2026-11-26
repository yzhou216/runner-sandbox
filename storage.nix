# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# NixOS virtual machines that test the object storage of geoai, whose tree is
# in geoai/.  `storage` runs Garage behind nginx, which serves its S3 API on
# port 443 and a directory of files on port 8443, both over TLS with a
# certificate from a test CA.  `client` trusts that CA through its system
# trust store and runs geoai.  The flake of geoai comes from the clone at
# WORKSPACE as a Git repository.
#
# Usage: nix build --impure --expr 'import ./storage.nix { workspace = "WORKSPACE"; }'
{ workspace }:
let
  geoai = builtins.getFlake "git+file://${workspace}?dir=geoai&shallow=1";
  system = "x86_64-linux";
  pkgs = geoai.inputs.nixpkgs.legacyPackages.${system};
  pga = ./geoai/tests/data/PGA_250y.tif;
  reference = ./geoai/tests/data/hazard_earthquake.tif;
  # Output of 32-bit numbers, 55 MB, which is uploaded in parts
  scaled = pkgs.writeText "scaled.scm" ''
    (define-hazard (scaled pga)
      (nodata -9999)
      (pixelwise (pga) (* pga 9.80665)))
  '';
  files = pkgs.runCommand "files" { } ''
    mkdir $out
    cp ${pga} $out/PGA_250y.tif
  '';
  certificates =
    pkgs.runCommand "certificates"
      {
        nativeBuildInputs = [ pkgs.openssl ];
        configuration = ''
          [req]
          distinguished_name = name
          [name]
          [ca]
          basicConstraints = critical, CA:TRUE
          keyUsage = critical, keyCertSign, cRLSign
          subjectKeyIdentifier = hash
          [storage]
          subjectAltName = DNS:storage
          basicConstraints = CA:FALSE
          keyUsage = critical, digitalSignature
          extendedKeyUsage = serverAuth
        '';
        passAsFile = [ "configuration" ];
      }
      ''
        mkdir $out
        cd $out
        openssl req -x509 -config $configurationPath -extensions ca -subj "/CN=geoai test CA" \
          -newkey ec -pkeyopt ec_paramgen_curve:P-256 -noenc -days 3650 -keyout ca.key -out ca.pem
        openssl req -new -config $configurationPath -subj /CN=storage \
          -newkey ec -pkeyopt ec_paramgen_curve:P-256 -noenc -keyout storage.key -out storage.csr
        openssl x509 -req -in storage.csr -CA ca.pem -CAkey ca.key -set_serial 1 -days 3650 \
          -extfile $configurationPath -extensions storage -out storage.pem
        openssl verify -CAfile ca.pem storage.pem
        rm ca.key storage.csr
      '';
  tls = {
    onlySSL = true;
    sslCertificate = "${certificates}/storage.pem";
    sslCertificateKey = "${certificates}/storage.key";
  };
in
pkgs.testers.runNixOSTest {
  name = "geoai-storage";

  nodes.storage = {
    services.garage = {
      enable = true;
      package = pkgs.garage_2;
      settings = {
        replication_factor = 1;
        consistency_mode = "consistent";
        rpc_bind_addr = "[::]:3901";
        rpc_public_addr = "127.0.0.1:3901";
        rpc_secret = "5c1915fa04d0b6739675c61bf5907eb0fe3d9c69850c83820f51b4d25d13868c";
        s3_api = {
          s3_region = "garage";
          api_bind_addr = "[::]:3900";
        };
      };
    };
    services.nginx = {
      enable = true;
      recommendedProxySettings = true;
      virtualHosts.storage = tls // {
        locations."/" = {
          proxyPass = "http://127.0.0.1:3900";
          extraConfig = ''
            client_max_body_size 0;
            proxy_request_buffering off;
          '';
        };
      };
      virtualHosts.files = tls // {
        serverName = "storage";
        listen = [
          {
            addr = "0.0.0.0";
            port = 8443;
            ssl = true;
          }
          {
            addr = "[::]";
            port = 8443;
            ssl = true;
          }
        ];
        root = files;
      };
    };
    networking.firewall.allowedTCPPorts = [
      443
      8443
    ];
    # Garage requires 1 GiB of free disk space
    virtualisation.diskSize = 4096;
  };

  nodes.client = {
    environment.systemPackages = [
      geoai.packages.${system}.default
      pkgs.s5cmd
      pkgs.gdalMinimal
    ];
    security.pki.certificateFiles = [ "${certificates}/ca.pem" ];
    nix.settings.experimental-features = [
      "nix-command"
      "flakes"
    ];
    virtualisation.memorySize = 4096;
    virtualisation.cores = 2;
    virtualisation.diskSize = 2048;
  };

  testScript = ''
    import re


    def field(name: str, text: str) -> str:
        match = re.search(rf"{name}:\s*(\S+)", text, re.IGNORECASE)
        assert match, text
        return match.group(1)


    def requests(pattern: str) -> list[str]:
        """Requests that nginx logged, as `"METHOD target HTTP/version" status`"""
        log = storage.succeed("cat /var/log/nginx/access.log")
        return re.findall(rf'"{pattern} HTTP/[\d.]+" \d+', log)


    start_all()

    with subtest("Garage serves a bucket through nginx"):
        storage.wait_for_unit("garage.service")
        storage.wait_for_open_port(3900)
        node = storage.succeed("garage node id").split("@")[0].strip()
        storage.succeed(f"garage layout assign -z vm -c 1G {node}")
        storage.succeed("garage layout apply --version 1")
        storage.succeed("garage bucket create hazards")
        key = storage.succeed("garage key create geoai")
        storage.succeed("garage bucket allow --read --write hazards --key geoai")
        storage.wait_for_open_port(443)
        storage.wait_for_open_port(8443)

    credentials = (
        f"AWS_REGION=garage AWS_ACCESS_KEY_ID={field('Key ID', key)} "
        f"AWS_SECRET_ACCESS_KEY={field('Secret key', key)}"
    )
    # Without a bundle named by the environment, the system trust store applies
    s3 = f"env -u SSL_CERT_FILE -u NIX_SSL_CERT_FILE AWS_ENDPOINT=https://storage {credentials}"
    s5cmd = f"{s3} s5cmd --endpoint-url https://storage"


    def assert_stored(url: str, expected: str) -> None:
        client.succeed(f"{s5cmd} cp {url} /tmp/download.tif")
        client.succeed(f"cmp {expected} /tmp/download.tif && rm /tmp/download.tif")


    client.wait_for_unit("multi-user.target")

    with subtest("An output written to the store matches one written to disk"):
        client.succeed("geoai run earthquake /tmp/earthquake.tif --input pga=${pga}")
        client.succeed(f"{s3} geoai run earthquake s3://hazards/runs/local.tif --input pga=${pga}")
        assert_stored("s3://hazards/runs/local.tif", "/tmp/earthquake.tif")

    with subtest("Inputs are read from the store and from a web server"):
        client.succeed(f"{s5cmd} cp ${pga} s3://hazards/sources/PGA_250y.tif")
        client.succeed(
            f"{s3} geoai run earthquake s3://hazards/runs/store.tif"
            " --input pga=s3://hazards/sources/PGA_250y.tif"
        )
        assert_stored("s3://hazards/runs/store.tif", "/tmp/earthquake.tif")
        client.succeed(
            f"{s3} geoai run earthquake s3://hazards/runs/web.tif"
            " --input pga=https://storage:8443/PGA_250y.tif"
        )
        assert_stored("s3://hazards/runs/web.tif", "/tmp/earthquake.tif")

    with subtest("The output takes the grid of an object given with --like"):
        client.succeed(f"{s5cmd} cp ${reference} s3://hazards/sources/hazard_earthquake.tif")
        client.succeed("geoai run earthquake /tmp/like.tif --input pga=${pga} --like ${reference}")
        client.fail("cmp /tmp/earthquake.tif /tmp/like.tif")
        client.succeed(
            f"{s3} geoai run earthquake s3://hazards/runs/like.tif"
            " --input pga=s3://hazards/sources/PGA_250y.tif"
            " --like s3://hazards/sources/hazard_earthquake.tif"
        )
        assert_stored("s3://hazards/runs/like.tif", "/tmp/like.tif")

    with subtest("A large output is uploaded in parts"):
        client.succeed("geoai run scaled /tmp/scaled.tif --hazards ${scaled} --input pga=${pga}")
        client.succeed(
            f"{s3} geoai run scaled s3://hazards/runs/scaled.tif --hazards ${scaled}"
            " --input pga=s3://hazards/sources/PGA_250y.tif"
        )
        assert_stored("s3://hazards/runs/scaled.tif", "/tmp/scaled.tif")
        parts = requests(r"PUT /hazards/runs/scaled\.tif\?\S*partNumber=\S*")
        assert len(parts) > 1 and all(part.endswith(" 200") for part in parts), parts
        completions = requests(r"POST /hazards/runs/scaled\.tif\?uploadId=\S*")
        assert [completion[-3:] for completion in completions] == ["200"], completions

    with subtest("geoai and GDAL describe an object as they do a file"):
        local = client.succeed("geoai info /tmp/scaled.tif")
        assert client.succeed(f"{s3} geoai info s3://hazards/runs/scaled.tif") == local, local
        info = client.succeed(
            "env AWS_S3_ENDPOINT=storage AWS_HTTPS=YES AWS_VIRTUAL_HOSTING=FALSE"
            f" CURL_CA_BUNDLE=/etc/ssl/certs/ca-certificates.crt {credentials}"
            " gdalinfo /vsis3/hazards/runs/scaled.tif"
        )
        assert "Size is 9825, 7964" in info and "Description = scaled" in info, info
        ranges = requests(r"GET /hazards/runs/scaled\.tif")
        assert any(read.endswith(" 206") for read in ranges), ranges

    with subtest("Failures name their cause"):
        missing = client.fail(f"{s3} geoai info s3://hazards/runs/missing.tif 2>&1")
        assert "not found" in missing, missing
        forbidden = client.fail(
            f"{s3} AWS_SECRET_ACCESS_KEY=wrong geoai info s3://hazards/runs/scaled.tif 2>&1"
        )
        assert "403" in forbidden, forbidden
        untrusted = client.fail(
            f"{s3} SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
            " geoai info s3://hazards/runs/scaled.tif 2>&1"
        )
        assert "certificate" in untrusted.lower(), untrusted
        print(missing, forbidden, untrusted, sep="")

    downloads = requests(r"GET /hazards/sources/PGA_250y\.tif")
    print(f"PGA_250y.tif was read from the store {len(downloads)} times: {downloads}")
  '';
}
