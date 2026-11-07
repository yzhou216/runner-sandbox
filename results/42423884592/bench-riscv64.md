# ubuntu-24.04-riscv

CPU(s):              4

## Kernels of the conversion, 4 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 4.91 | 2.92 | 5.61 |
| 640x360 | pairs | yes | 4.36 | 2.30 | 5.05 |
| 640x360 | lanes | yes | 7.41 | 3.96 | 8.31 |
| 1280x720 | now | yes | 19.45 | 10.18 | 20.62 |
| 1280x720 | pairs | yes | 17.10 | 8.88 | 18.26 |
| 1280x720 | lanes | yes | 30.23 | 15.83 | 31.61 |
| 1920x1080 | now | yes | 43.98 | 22.57 | 45.68 |
| 1920x1080 | pairs | yes | 38.63 | 20.11 | 40.09 |
| 1920x1080 | lanes | yes | 68.20 | 34.35 | 69.48 |
| 3840x2160 | now | yes | 183.23 | 80.22 | 177.76 |
| 3840x2160 | pairs | yes | 156.85 | 69.47 | 154.72 |
| 3840x2160 | lanes | yes | 271.95 | 116.04 | 272.93 |

## Passes through whole clips, 4 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p | now, now | 180 | 7.7 | 1.06 |
| 1080p | pipelined, now | 180 | 9.3 | 1.32 |
| 1080p | threaded, now | 180 | 15.0 | 2.47 |
| 1080p | now, lanes | 180 | 7.2 | 1.13 |
| 1080p | pipelined, lanes | 180 | 9.2 | 1.53 |
| 1080p | threaded, lanes | 180 | 14.3 | 2.59 |
