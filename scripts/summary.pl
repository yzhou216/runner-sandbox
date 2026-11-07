#!/usr/bin/env perl
# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Prints the derivations whose builds took longest in a log that timed.pl
# wrote, from the first line of each build's output to its last, and how
# many derivations built and over how long
use strict;
use warnings;

my (%first, %last, $start, $end);
while (<>) {
    next unless /^(\d+) \S+ (.*)$/;
    $start //= $1;
    $end = $1;
    next unless $2 =~ /^([^ >]+)> /;
    $first{$1} //= $end;
    $last{$1} = $end;
}
my @names = sort { ($last{$b} - $first{$b}) <=> ($last{$a} - $first{$a}) } keys %first;
printf "%d derivations built over %d seconds\n", scalar @names, ($end // 0) - ($start // 0);
printf "%7d s  %s\n", $last{$_} - $first{$_}, $_ for @names[0 .. ($#names < 24 ? $#names : 24)];
