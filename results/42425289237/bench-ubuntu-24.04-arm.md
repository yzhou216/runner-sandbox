# ubuntu-24.04-arm

CPU(s):                                  4
Model name:                              Neoverse-N2

## Kernels of the conversion, 4 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 0.53 | 0.16 | 0.61 |
| 640x360 | pairs | yes | 0.27 | 0.11 | 0.37 |
| 640x360 | lanes | yes | 0.24 | 0.09 | 0.33 |
| 1280x720 | now | yes | 2.13 | 0.60 | 2.23 |
| 1280x720 | pairs | yes | 1.12 | 0.34 | 1.24 |
| 1280x720 | lanes | yes | 1.01 | 0.32 | 1.11 |
| 1920x1080 | now | yes | 4.80 | 1.35 | 4.96 |
| 1920x1080 | pairs | yes | 2.56 | 0.78 | 2.71 |
| 1920x1080 | lanes | yes | 2.28 | 0.71 | 2.41 |
| 3840x2160 | now | yes | 19.18 | 5.26 | 19.34 |
| 3840x2160 | pairs | yes | 10.18 | 2.98 | 10.38 |
| 3840x2160 | lanes | yes | 9.14 | 2.72 | 9.26 |

## cinq's conversion of high definition, at ec61ead and in the snapshot

| size | ec61ead ms | ec61ead CPU ms | snapshot ms | snapshot CPU ms | same bytes |
| --- | --- | --- | --- | --- | --- |
| 1280x720 | 0.69 | 2.41 | 0.42 | 1.30 | yes |
| 1920x1080 | 1.42 | 5.09 | 0.80 | 2.63 | yes |
| 3840x2160 | 5.34 | 19.55 | 2.86 | 9.70 | yes |

## Passes through whole clips, 4 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p | now, now | 180 | 81.4 | 1.26 |
| 1080p | now, pairs | 180 | 87.9 | 1.15 |
| 1080p | pipelined, pairs | 180 | 98.7 | 1.27 |
| 1080p | threaded, now | 180 | 206.6 | 3.37 |
| 1080p | threaded, pairs | 180 | 228.8 | 3.21 |
| 1080p30-grain | now, now | 180 | 42.6 | 1.14 |
| 1080p30-grain | now, pairs | 180 | 43.5 | 1.08 |
| 1080p30-grain | pipelined, pairs | 180 | 42.5 | 1.13 |
| 1080p30-grain | threaded, now | 180 | 99.9 | 2.81 |
| 1080p30-grain | threaded, pairs | 180 | 104.7 | 2.71 |
| 1080p60-grain | now, now | 240 | 59.8 | 1.19 |
| 1080p60-grain | now, pairs | 240 | 62.6 | 1.10 |
| 1080p60-grain | pipelined, pairs | 240 | 68.6 | 1.19 |
| 1080p60-grain | threaded, now | 240 | 142.7 | 3.01 |
| 1080p60-grain | threaded, pairs | 240 | 151.4 | 2.85 |
| 2160p | now, now | 120 | 21.2 | 1.30 |
| 2160p | now, pairs | 120 | 22.5 | 1.16 |
| 2160p | pipelined, pairs | 120 | 24.7 | 1.26 |
| 2160p | threaded, now | 120 | 51.9 | 3.34 |
| 2160p | threaded, pairs | 120 | 57.4 | 3.16 |
| 2160p30-grain | now, now | 90 | 14.0 | 1.20 |
| 2160p30-grain | now, pairs | 90 | 14.3 | 1.11 |
| 2160p30-grain | pipelined, pairs | 90 | 15.2 | 1.16 |
| 2160p30-grain | threaded, now | 90 | 30.7 | 2.77 |
| 2160p30-grain | threaded, pairs | 90 | 32.6 | 2.65 |

