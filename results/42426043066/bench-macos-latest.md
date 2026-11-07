# macos-latest

Apple M1 (Virtual)
3

## Kernels of the conversion, 3 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 0.53 | 0.37 | 0.60 |
| 640x360 | pairs | yes | 0.14 | 0.12 | 0.21 |
| 640x360 | lanes | yes | 0.12 | 0.07 | 0.18 |
| 1280x720 | now | yes | 2.17 | 0.72 | 2.17 |
| 1280x720 | pairs | yes | 0.56 | 0.23 | 0.65 |
| 1280x720 | lanes | yes | 0.51 | 0.25 | 0.60 |
| 1920x1080 | now | yes | 4.94 | 1.75 | 5.23 |
| 1920x1080 | pairs | yes | 1.36 | 0.51 | 1.50 |
| 1920x1080 | lanes | yes | 1.25 | 0.46 | 1.34 |
| 3840x2160 | now | yes | 20.93 | 7.39 | 20.42 |
| 3840x2160 | pairs | yes | 5.62 | 2.59 | 5.88 |
| 3840x2160 | lanes | yes | 5.09 | 2.32 | 5.15 |

## cinq's conversion of high definition, at ec61ead and in the snapshot

| size | ec61ead ms | ec61ead CPU ms | snapshot ms | snapshot CPU ms | same bytes |
| --- | --- | --- | --- | --- | --- |
| 1280x720 | 0.75 | 2.15 | 0.24 | 0.63 | yes |
| 1920x1080 | 1.66 | 4.74 | 0.50 | 1.32 | yes |
| 3840x2160 | 6.79 | 19.06 | 2.22 | 5.46 | yes |

