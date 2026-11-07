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
| 640x360 | pairs | yes | 0.27 | 0.10 | 0.37 |
| 640x360 | lanes | yes | 0.24 | 0.16 | 0.36 |
| 1280x720 | now | yes | 2.14 | 0.61 | 2.25 |
| 1280x720 | pairs | yes | 1.14 | 0.36 | 1.26 |
| 1280x720 | lanes | yes | 1.02 | 0.58 | 1.18 |
| 1920x1080 | now | yes | 4.79 | 1.34 | 4.91 |
| 1920x1080 | pairs | yes | 2.57 | 0.80 | 2.75 |
| 1920x1080 | lanes | yes | 2.29 | 0.72 | 2.43 |
| 3840x2160 | now | yes | 19.18 | 5.31 | 19.38 |
| 3840x2160 | pairs | yes | 10.29 | 3.06 | 10.49 |
| 3840x2160 | lanes | yes | 9.19 | 2.75 | 9.30 |

## cinq's conversion of high definition, at ec61ead and in the snapshot

| size | ec61ead ms | ec61ead CPU ms | snapshot ms | snapshot CPU ms | same bytes |
| --- | --- | --- | --- | --- | --- |
| 1280x720 | 0.72 | 2.46 | 0.45 | 1.36 | yes |
| 1920x1080 | 1.45 | 5.17 | 0.84 | 2.71 | yes |
| 3840x2160 | 5.37 | 19.61 | 2.94 | 9.91 | yes |

