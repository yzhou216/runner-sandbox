@echo off
rem SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
rem
rem SPDX-License-Identifier: 0BSD OR CC0-1.0

rem Tests that [GuiRunOnce] starts after setup, one at a time through
rem RUNNER.EXE, which logs each result to COM1.  keys.sh types into cinq's
rem window while the second runs.
set T=%~dp0
mkdir C:\results
%T%RUNNER.EXE 30 CINQ-version %T%CINQ.EXE --version
%T%RUNNER.EXE 150 CINQ %T%CINQ.EXE
echo ROSTEST done
ping -n 5 127.0.0.1 > nul
shutdown -s -t 0
