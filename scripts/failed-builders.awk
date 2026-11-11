# SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
#
# SPDX-License-Identifier: 0BSD OR CC0-1.0

# Derivations whose own builders failed, rather than one of their dependencies
/Cannot build '/ { split($0, part, "'"); drv = part[2] }
/Reason: builder failed/ { print drv }
