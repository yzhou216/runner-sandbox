<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# cinq: sound in the kernels of the runners

This branch looks, on each image of GNU/Linux runners, for ALSA in the
kernel and for the module of its loopback card, snd-aloop, which tests of
the hw devices of ALSA would need.

## Results

Run 43573188437 passed on ubuntu-latest, ubuntu-24.04, ubuntu-22.04,
ubuntu-24.04-arm, and ubuntu-22.04-arm.

- No kernel of them has `CONFIG_SND`: neither the Azure kernels 7.0 and
  6.8 nor the AWS kernel 7.0 of ubuntu-24.04.
- The images of 22.04 offer `linux-modules-extra` for their kernels, but
  it holds no snd-aloop; those of 24.04 offer no such package.
- Tests of ALSA's hw devices therefore run in a virtual machine, as the
  branch `cinq/2026-11-13-linux-quirks` runs them in NixOS.
