# ubuntu-latest, x86-64-v3

## Kernels of the conversion, 4 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 1.86 | 0.63 | 2.51 |
| 640x360 | pairs | yes | 0.17 | 0.11 | 0.44 |
| 640x360 | lanes | yes | 0.11 | 0.07 | 0.29 |
| 1280x720 | now | yes | 7.47 | 2.51 | 9.81 |
| 1280x720 | pairs | yes | 0.71 | 0.41 | 1.47 |
| 1280x720 | lanes | yes | 0.44 | 0.27 | 0.95 |
| 1920x1080 | now | yes | 16.89 | 5.66 | 21.86 |
| 1920x1080 | pairs | yes | 1.60 | 0.88 | 3.04 |
| 1920x1080 | lanes | yes | 1.00 | 0.58 | 1.92 |
| 3840x2160 | now | yes | 68.62 | 23.49 | 88.01 |
| 3840x2160 | pairs | yes | 7.05 | 4.19 | 12.31 |
| 3840x2160 | lanes | yes | 4.74 | 2.93 | 7.99 |

