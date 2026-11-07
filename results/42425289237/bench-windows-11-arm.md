# windows-11-arm

Cobalt 100
4

## Kernels of the conversion, 4 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 0.52 | 0.17 | 0.78 |
| 640x360 | pairs | yes | 0.27 | 0.10 | 0.39 |
| 640x360 | lanes | yes | 0.24 | 0.09 | 0.39 |
| 1280x720 | now | yes | 2.67 | 1.08 | 3.16 |
| 1280x720 | pairs | yes | 1.68 | 0.86 | 2.54 |
| 1280x720 | lanes | yes | 1.53 | 0.83 | 2.70 |
| 1920x1080 | now | yes | 6.11 | 2.18 | 6.28 |
| 1920x1080 | pairs | yes | 3.83 | 1.61 | 3.98 |
| 1920x1080 | lanes | yes | 3.51 | 1.54 | 4.80 |
| 3840x2160 | now | yes | 22.99 | 7.18 | 24.21 |
| 3840x2160 | pairs | yes | 14.23 | 4.96 | 14.85 |
| 3840x2160 | lanes | yes | 13.08 | 4.69 | 13.60 |

## cinq's conversion of high definition, at ec61ead and in the snapshot

| size | ec61ead ms | ec61ead CPU ms | snapshot ms | snapshot CPU ms | same bytes |
| --- | --- | --- | --- | --- | --- |
| 1280x720 | 1.29 | 3.79 | 1.03 | 2.97 | yes |
| 1920x1080 | 2.42 | 7.45 | 1.79 | 4.80 | yes |
| 3840x2160 | 7.44 | 23.00 | 4.95 | 14.84 | yes |

## Passes through whole clips, 4 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p | now, now | 180 | 72.7 | 1.33 |
| 1080p | now, pairs | 180 | 76.2 | 1.22 |
| 1080p | pipelined, pairs | 180 | 79.0 | 1.26 |
| 1080p | threaded, now | 180 | 188.8 | 3.52 |
| 1080p | threaded, pairs | 180 | 200.2 | 3.28 |
| 1080p30-grain | now, now | 180 | 38.8 | 1.20 |
| 1080p30-grain | now, pairs | 180 | 39.7 | 1.09 |
| 1080p30-grain | pipelined, pairs | 180 | 40.3 | 1.14 |
| 1080p30-grain | threaded, now | 180 | 92.2 | 2.93 |
| 1080p30-grain | threaded, pairs | 180 | 95.8 | 2.81 |
| 1080p60-grain | now, now | 240 | 53.3 | 1.23 |
| 1080p60-grain | now, pairs | 240 | 55.1 | 1.19 |
| 1080p60-grain | pipelined, pairs | 240 | 56.7 | 1.16 |
| 1080p60-grain | threaded, now | 240 | 126.3 | 2.98 |
| 1080p60-grain | threaded, pairs | 240 | 134.9 | 2.93 |
| 2160p | now, now | 120 | 18.5 | 1.30 |
| 2160p | now, pairs | 120 | 19.4 | 1.22 |
| 2160p | pipelined, pairs | 120 | 20.0 | 1.25 |
| 2160p | threaded, now | 120 | 47.5 | 3.36 |
| 2160p | threaded, pairs | 120 | 51.3 | 3.21 |
| 2160p30-grain | now, now | 90 | 12.4 | 1.18 |
| 2160p30-grain | now, pairs | 90 | 12.8 | 1.15 |
| 2160p30-grain | pipelined, pairs | 90 | 13.1 | 1.14 |
| 2160p30-grain | threaded, now | 90 | 27.4 | 2.85 |
| 2160p30-grain | threaded, pairs | 90 | 29.4 | 2.69 |

