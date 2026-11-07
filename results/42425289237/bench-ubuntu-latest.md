# ubuntu-latest

CPU(s):                                  4
Model name:                              AMD EPYC 9V74 80-Core Processor

## Kernels of the conversion, 4 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 1.80 | 0.58 | 2.33 |
| 640x360 | pairs | yes | 0.45 | 0.24 | 0.95 |
| 640x360 | lanes | yes | 0.41 | 0.20 | 0.81 |
| 1280x720 | now | yes | 7.21 | 2.33 | 9.17 |
| 1280x720 | pairs | yes | 1.79 | 1.01 | 3.72 |
| 1280x720 | lanes | yes | 1.65 | 0.96 | 3.38 |
| 1920x1080 | now | yes | 16.30 | 5.23 | 20.48 |
| 1920x1080 | pairs | yes | 4.05 | 2.24 | 8.20 |
| 1920x1080 | lanes | yes | 3.73 | 2.12 | 7.41 |
| 3840x2160 | now | yes | 65.49 | 21.17 | 81.93 |
| 3840x2160 | pairs | yes | 16.56 | 9.24 | 32.28 |
| 3840x2160 | lanes | yes | 15.53 | 8.85 | 29.25 |

## cinq's conversion of high definition, at ec61ead and in the snapshot

| size | ec61ead ms | ec61ead CPU ms | snapshot ms | snapshot CPU ms | same bytes |
| --- | --- | --- | --- | --- | --- |
| 1280x720 | 2.40 | 9.30 | 1.14 | 4.28 | yes |
| 1920x1080 | 5.30 | 20.66 | 2.49 | 9.45 | yes |
| 3840x2160 | 21.26 | 82.67 | 10.05 | 37.69 | yes |

## Passes through whole clips, 4 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p | now, now | 180 | 72.5 | 1.62 |
| 1080p | now, pairs | 180 | 78.1 | 1.46 |
| 1080p | pipelined, pairs | 180 | 85.9 | 1.64 |
| 1080p | threaded, now | 180 | 126.0 | 3.53 |
| 1080p | threaded, pairs | 180 | 144.4 | 3.45 |
| 1080p30-grain | now, now | 180 | 33.5 | 1.28 |
| 1080p30-grain | now, pairs | 180 | 34.7 | 1.18 |
| 1080p30-grain | pipelined, pairs | 180 | 36.1 | 1.27 |
| 1080p30-grain | threaded, now | 180 | 63.0 | 3.06 |
| 1080p30-grain | threaded, pairs | 180 | 66.9 | 2.90 |
| 1080p60-grain | now, now | 240 | 48.4 | 1.42 |
| 1080p60-grain | now, pairs | 240 | 50.8 | 1.27 |
| 1080p60-grain | pipelined, pairs | 240 | 53.7 | 1.40 |
| 1080p60-grain | threaded, now | 240 | 86.4 | 3.18 |
| 1080p60-grain | threaded, pairs | 240 | 93.6 | 3.06 |
| 2160p | now, now | 120 | 17.7 | 1.60 |
| 2160p | now, pairs | 120 | 18.9 | 1.44 |
| 2160p | pipelined, pairs | 120 | 21.1 | 1.61 |
| 2160p | threaded, now | 120 | 32.8 | 3.60 |
| 2160p | threaded, pairs | 120 | 36.6 | 3.45 |
| 2160p30-grain | now, now | 90 | 11.0 | 1.38 |
| 2160p30-grain | now, pairs | 90 | 11.7 | 1.26 |
| 2160p30-grain | pipelined, pairs | 90 | 12.3 | 1.35 |
| 2160p30-grain | threaded, now | 90 | 20.2 | 3.07 |
| 2160p30-grain | threaded, pairs | 90 | 21.3 | 2.84 |

