# ubuntu-latest

CPU(s):                                  4
Model name:                              AMD EPYC 7763 64-Core Processor

## Kernels of the conversion, 4 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 2.05 | 0.68 | 2.72 |
| 640x360 | pairs | yes | 0.52 | 0.27 | 1.09 |
| 640x360 | lanes | yes | 0.48 | 0.23 | 0.94 |
| 1280x720 | now | yes | 8.25 | 4.42 | 10.56 |
| 1280x720 | pairs | yes | 2.07 | 1.08 | 4.05 |
| 1280x720 | lanes | yes | 1.92 | 0.93 | 3.46 |
| 1920x1080 | now | yes | 18.58 | 6.11 | 23.80 |
| 1920x1080 | pairs | yes | 4.66 | 2.39 | 8.86 |
| 1920x1080 | lanes | yes | 4.33 | 2.04 | 7.60 |
| 3840x2160 | now | yes | 75.34 | 25.07 | 95.10 |
| 3840x2160 | pairs | yes | 19.25 | 10.04 | 35.95 |
| 3840x2160 | lanes | yes | 17.96 | 8.64 | 30.68 |

## cinq's conversion of high definition, at ec61ead and in the snapshot

| size | ec61ead ms | ec61ead CPU ms | snapshot ms | snapshot CPU ms | same bytes |
| --- | --- | --- | --- | --- | --- |
| 1280x720 | 2.81 | 10.78 | 1.36 | 4.96 | yes |
| 1920x1080 | 6.17 | 23.91 | 2.92 | 10.95 | yes |
| 3840x2160 | 24.93 | 95.43 | 11.97 | 43.86 | yes |

