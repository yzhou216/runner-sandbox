# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Can the GHC bootstrap chain skip a rung?  andreabedini/ghc-bootstrap climbs
# 6.2.2 -> 6.6 -> 6.10.4 -> 6.12.3, but configure.ac lets 6.8.3 take any boot
# compiler and 6.12.3 take 6.8 or later.  So try 6.2.2 -> 6.8.3 -> 6.12.3 with
# the build settings the chain uses for 6.6 and 6.12.3 (toolchain flags for
# 2008 and 2010 C, unregisterised 6.8.3, no GHCi) and no source changes.  The
# tarballs' generated parsers are kept.
#
#   ghc622   haskell.compiler.ghc622Bootstrap of nixpkgs#591060
#   ghc683   GHC 6.8.3 built by ghc622
#   ghc6123  GHC 6.12.3 built by ghc683
let
  nixpkgs = builtins.fetchTarball {
    url = "https://github.com/yzhou216/nixpkgs/archive/30d01b1a222279606ad447a7381b0ff6c26edee9.tar.gz";
    sha256 = "143f5jv868bf8rv6rf76v6rkzyl3hxcvkp9yrys1k2pid1hdg9a5";
  };
  pkgs = import nixpkgs { };
  inherit (pkgs)
    stdenv
    fetchurl
    perl
    ncurses
    ;

  ghc622 = pkgs.haskell.compiler.ghc622Bootstrap;
  inherit (ghc622) gmp nonPie;

  # Flags for the C that GHC writes and the C in its tree, as the chain gives
  # 6.6 and 6.12.3 for gcc 16
  cflags = "-std=gnu89 -fpermissive -w -fwrapv";
  optc = "-optc-std=gnu89 -optc-fpermissive -optc-w -optc-fwrapv";

  # The installed compiler compiles and runs a program on its own
  check = version: ''
    mkdir t
    cat > t/Main.hs <<'EOF'
    main :: IO ()
    main = do
      putStrLn "hello"
      let zs = [fromIntegral i * 2 ^ (70 :: Int) * 0 | i <- [1 .. 300000 :: Int]] :: [Integer]
      print (product [1 .. 25 :: Integer], length zs, sum zs)
    EOF
    (cd t && $out/bin/ghc -v0 -O --make Main.hs -o main && ./main > out.txt)
    printf 'hello\n(15511210043330985984000000,300000,0)\n' | diff - t/out.txt
    $out/bin/ghc --numeric-version | grep -qx ${version}
  '';
in
rec {
  inherit ghc622;

  ghc683 = stdenv.mkDerivation {
    pname = "ghc";
    version = "6.8.3";

    src = fetchurl {
      url = "https://downloads.haskell.org/~ghc/6.8.3/ghc-6.8.3-src.tar.bz2";
      hash = "sha256-H8H/gqVVUy8cnS3GKP2d5ebrqyzm7pSQo0F0zrb3bms=";
    };

    nativeBuildInputs = [
      perl
      ghc622
    ];
    buildInputs = [ gmp ];

    hardeningDisable = [ "all" ];
    env = nonPie;

    postPatch = ''
      patchShebangs .
      cat > mk/build.mk <<'EOF'
      GhcUnregisterised = YES
      GhcWithNativeCodeGen = NO
      GhcWithInterpreter = NO
      GhcThreaded = NO
      GhcWithSMP = NO
      SplitObjs = NO
      GhcLibWays =
      GhcRTSWays =
      XMLDocWays =
      HADDOCK_DOCS = NO
      GhcLibHcOpts = -O ${optc}
      GhcStage2HcOpts = ${optc}
      SRC_HC_OPTS += ${optc}
      SRC_CC_OPTS += ${cflags}
      SRC_HSC2HS_OPTS += --cflag=-std=gnu89 --cflag=-fpermissive --cflag=-w
      EOF
    '';

    configurePhase = ''
      runHook preConfigure
      ./configure --prefix=$out --with-ghc=${ghc622}/bin/ghc --with-gcc=gcc CFLAGS="${cflags}"
      runHook postConfigure
    '';

    preBuild = "ulimit -s unlimited";
    enableParallelBuilding = false;
    dontStrip = true;

    doInstallCheck = true;
    installCheckPhase = check "6.8.3";

    passthru = { inherit nonPie; };
  };

  ghc6123 = stdenv.mkDerivation {
    pname = "ghc";
    version = "6.12.3";

    src = fetchurl {
      url = "https://downloads.haskell.org/~ghc/6.12.3/ghc-6.12.3-src.tar.bz2";
      hash = "sha256-bL2+QVAR8sfRXk2FB1jY05P3Bhe4jLMjfSxgK7YOXmg=";
    };

    nativeBuildInputs = [
      perl
      ghc683
    ];
    buildInputs = [
      gmp
      ncurses
    ];

    hardeningDisable = [ "all" ];
    env = nonPie;

    postPatch = ''
      patchShebangs .
      # As the chain's 6.12.3: includes/Rts.h has `#if __GNUC__ > 4` blocks
      # that no 2010 gcc took, one of them not valid for any cpp
      sed -i 's/^#if __GNUC__ > 4$/#if 0/' includes/Rts.h
      cat > mk/build.mk <<'EOF'
      GhcUnregisterised = NO
      GhcWithNativeCodeGen = YES
      GhcWithInterpreter = NO
      GhcWithSMP = NO
      GhcLibWays = v
      GhcRTSWays =
      SplitObjs = NO
      HADDOCK_DOCS = NO
      HSCOLOUR_SRCS = NO
      BUILD_DOCBOOK_HTML = NO
      BUILD_DOCBOOK_PS = NO
      BUILD_DOCBOOK_PDF = NO
      LATEX_DOCS = NO
      GhcStage1HcOpts = -O
      GhcStage2HcOpts = -O -fasm
      GhcLibHcOpts = -O -fasm
      SRC_HC_OPTS = -H64m -O ${optc}
      SRC_CC_OPTS += ${cflags}
      SRC_HSC2HS_OPTS += --cflag=-std=gnu89 --cflag=-fpermissive --cflag=-w
      EOF
    '';

    configurePhase = ''
      runHook preConfigure
      ./configure --prefix=$out --with-ghc=${ghc683}/bin/ghc --with-gcc=gcc CFLAGS="-std=gnu89 -fpermissive -w"
      runHook postConfigure
    '';

    preBuild = "ulimit -s unlimited";
    enableParallelBuilding = false;
    dontStrip = true;

    doInstallCheck = true;
    installCheckPhase = check "6.12.3";
  };
}
