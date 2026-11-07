# macos-latest

Apple M1 (Virtual)
3

## Kernels of the conversion, 3 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 0.51 | 0.20 | 0.58 |
| 640x360 | pairs | yes | 0.14 | 0.14 | 0.19 |
| 640x360 | lanes | yes | 0.13 | 0.07 | 0.20 |
| 1280x720 | now | yes | 2.33 | 1.18 | 2.23 |
| 1280x720 | pairs | yes | 0.60 | 0.43 | 0.65 |
| 1280x720 | lanes | yes | 0.51 | 0.36 | 0.59 |
| 1920x1080 | now | yes | 4.79 | 1.66 | 4.82 |
| 1920x1080 | pairs | yes | 1.17 | 0.53 | 1.45 |
| 1920x1080 | lanes | yes | 1.19 | 0.45 | 1.25 |
| 3840x2160 | now | yes | 18.18 | 6.84 | 19.27 |
| 3840x2160 | pairs | yes | 5.01 | 2.25 | 5.68 |
| 3840x2160 | lanes | yes | 4.70 | 2.08 | 4.97 |

## cinq's conversion of high definition, at ec61ead and in the snapshot

| size | ec61ead ms | ec61ead CPU ms | snapshot ms | snapshot CPU ms | same bytes |
| --- | --- | --- | --- | --- | --- |
| 1280x720 | 0.79 | 2.21 | 0.26 | 0.64 | yes |
| 1920x1080 | 1.70 | 4.82 | 0.53 | 1.37 | yes |
| 3840x2160 | 6.91 | 19.49 | 3.91 | 5.59 | yes |

## Passes through whole clips, 3 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p | now, now | 180 | 101.4 | 1.32 |
| 1080p | now, pairs | 180 | 114.7 | 1.10 |
| 1080p | pipelined, pairs | 180 | 121.5 | 1.19 |
| 1080p | threaded, now | 180 | 161.5 | 2.73 |
| 1080p | threaded, pairs | 180 | 223.5 | 2.49 |
| 1080p30-grain | now, now | 180 | 48.6 | 1.14 |
| 1080p30-grain | now, pairs | 180 | 52.6 | 1.04 |
| 1080p30-grain | pipelined, pairs | 180 | 53.4 | 1.09 |
| 1080p30-grain | threaded, now | 180 | 70.7 | 2.21 |
| 1080p30-grain | threaded, pairs | 180 | 71.5 | 2.07 |
| 1080p60-grain | now, now | 240 | 59.6 | 1.24 |
| 1080p60-grain | now, pairs | 240 | 59.6 | 1.07 |
| 1080p60-grain | pipelined, pairs | 240 | 71.9 | 1.18 |
| 1080p60-grain | threaded, now | 240 | 109.1 | 2.17 |
| 1080p60-grain | threaded, pairs | 240 | 122.6 | 2.03 |
| 2160p | now, now | 120 | 22.1 | 1.31 |
| 2160p | now, pairs | 120 | 24.9 | 1.09 |
| 2160p | pipelined, pairs | 120 | 26.8 | 1.21 |
| 2160p | threaded, now | 120 | 42.9 | 2.51 |
| 2160p | threaded, pairs | 120 | 44.2 | 2.26 |
| 2160p30-grain | now, now | 90 | 15.7 | 1.18 |
| 2160p30-grain | now, pairs | 90 | 17.3 | 1.05 |
| 2160p30-grain | pipelined, pairs | 90 | 18.4 | 1.10 |
| 2160p30-grain | threaded, now | 90 | 26.9 | 2.13 |
| 2160p30-grain | threaded, pairs | 90 | 30.4 | 1.91 |

