#!/bin/bash
# SPDX-License-Identifier: GPL-2.0-only
# Adapted from AFK-surf/xnulinux scripts/vm-xnu-rebuild.sh, which builds
# xnulinux's own kexts only when Apple's macOS 27.2 kernelcache is present:
# XnuCPU, XnuDisk and XnuDrm, the kexts of the free kernel collection
# (scripts/free-kc/build.sh), built against the k27 kernel's objects.
#   kexts.sh XNULINUX
set -euo pipefail
POC=$1
cd ~/w/dvm
export PATH=$PATH:/usr/lib/llvm-21/bin
NETINC="$POC/build/netinc"; NETSRC="$POC/build/IONetworkingFamily-188"; mkdir -p "$NETINC/IOKit"
[ -e "$NETSRC/IOEthernetController.h" ] || git clone -q --depth 1 --branch IONetworkingFamily-188 https://github.com/apple-oss-distributions/IONetworkingFamily.git "$NETSRC"
ln -sfn "$NETSRC" "$NETINC/IOKit/network"
for h in toolchains/sdks/*.sdk/System/Library/Frameworks/Kernel.framework/Headers/IOKit/hid; do
  [ -e "$h/IOHIDDevice.h" ] || continue
  ln -sfn "$PWD/$h" "$NETINC/IOKit/hid"; ln -sfn "$PWD/$h/../hidsystem" "$NETINC/IOKit/hidsystem"
  ln -sfn "$PWD/$h/../graphics" "$NETINC/IOKit/graphics"; ln -sfn "$PWD/$h/../storage" "$NETINC/IOKit/storage"
  ln -sfn "$PWD/${h%%/System/*}/usr/include/TargetConditionals.h" "$NETINC/TargetConditionals.h"
  ln -sfn "$PWD/$h/../../HIDDriverKit" "$NETINC/HIDDriverKit"
  break
done
for n in XnuCPU XnuDisk XnuDrm; do
  python3 scripts/build-native-kext.py "$n" "$POC/kexts/$n/$n.cpp" -I "$NETINC" --obj k27-obj
done
