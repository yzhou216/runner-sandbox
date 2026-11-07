# windows-11-arm

Cobalt 100
4

## Kernels of the conversion, 4 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 0.52 | 0.16 | 0.59 |
| 640x360 | pairs | yes | 0.28 | 0.10 | 0.35 |
| 640x360 | lanes | yes | 0.25 | 0.09 | 0.31 |
| 1280x720 | now | yes | 2.67 | 1.07 | 2.77 |
| 1280x720 | pairs | yes | 1.68 | 0.84 | 1.99 |
| 1280x720 | lanes | yes | 1.55 | 0.81 | 2.23 |
| 1920x1080 | now | yes | 6.00 | 2.14 | 5.91 |
| 1920x1080 | pairs | yes | 3.71 | 1.59 | 4.26 |
| 1920x1080 | lanes | yes | 3.47 | 1.51 | 4.06 |
| 3840x2160 | now | yes | 22.87 | 7.20 | 26.74 |
| 3840x2160 | pairs | yes | 13.80 | 4.93 | 14.67 |
| 3840x2160 | lanes | yes | 12.76 | 4.64 | 14.99 |

## cinq's conversion of high definition, at ec61ead and in the snapshot

| size | ec61ead ms | ec61ead CPU ms | snapshot ms | snapshot CPU ms | same bytes |
| --- | --- | --- | --- | --- | --- |
| 1280x720 | 1.25 | 3.48 | 0.98 | 2.89 | yes |
| 1920x1080 | 2.35 | 7.41 | 1.74 | 4.34 | yes |
| 3840x2160 | 7.34 | 23.80 | 4.93 | 14.52 | yes |

