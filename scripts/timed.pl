#!/usr/bin/env perl
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Copies standard input to the file that the first argument names, each
# line stamped with the time, and prints the lines that are not the output
# of a build, along with the first line of each derivation's build, so that
# the log of a job shows how far Nix has come without the logs of builds
use strict;
use warnings;
use IO::Handle;
use POSIX qw(strftime);

open(my $log, '>', $ARGV[0]) or die "$ARGV[0]: $!";
$log->autoflush(1);
STDOUT->autoflush(1);
my %seen;
while (my $line = <STDIN>) {
    my $now = time;
    my $stamped = $now . strftime(' %T ', gmtime $now) . $line;
    print $log $stamped;
    if ($line =~ /^([^ >]+)> /) {
        print $stamped unless $seen{$1}++;
    } else {
        print $stamped;
    }
}
