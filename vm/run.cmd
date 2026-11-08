@echo off
rem SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
rem
rem SPDX-License-Identifier: 0BSD OR CC0-1.0

rem Tests that [GuiRunOnce] starts after setup, one at a time through
rem RUNNER.EXE, which logs each result to COM1 and to C:\results
set R=C:\results
set T=%~dp0
mkdir %R%
ver > %R%\ver.txt
%T%RUNNER.EXE 30 TLSPROBE %T%TLSPROBE.EXE
%T%RUNNER.EXE 30 ONE_SH %T%ONE_SH.EXE
%T%RUNNER.EXE 30 HELLO_SH %T%HELLO_SH.EXE
for %%m in (empty sleep mutex print local drop) do (
  %T%RUNNER.EXE 30 THRDS_SH-%%m %T%THRDS_SH.EXE %%m
)
%T%RUNNER.EXE 30 CINQ_SH-version %T%CINQ_SH.EXE --version
%T%RUNNER.EXE 40 COUNT_SH %T%COUNT_SH.EXE
set ICED_BACKEND=tiny-skia
%T%RUNNER.EXE 40 COUNT_SH-tiny-skia %T%COUNT_SH.EXE
set ICED_BACKEND=
set ROSTEST_EMBEDDED_FONT=1
%T%RUNNER.EXE 40 COUNT_SH-font %T%COUNT_SH.EXE
set ROSTEST_EMBEDDED_FONT=
%T%RUNNER.EXE 60 CINQ_SH %T%CINQ_SH.EXE
for %%p in (ONE HELLO COUNT CINQ) do (
  %T%RUNNER.EXE 15 %%p_ST %T%%%p_ST.EXE
)
echo ROSTEST done
ping -n 5 127.0.0.1 > nul
shutdown -s -t 0
