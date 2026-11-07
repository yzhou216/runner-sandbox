# windows-latest, x86-64-v3

## Kernels of the conversion, 4 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 1.51 | 0.50 | 2.15 |
| 640x360 | pairs | yes | 0.12 | 0.23 | 0.55 |
| 640x360 | lanes | yes | 0.11 | 0.12 | 0.23 |
| 1280x720 | now | yes | 5.85 | 2.04 | 8.07 |
| 1280x720 | pairs | yes | 0.78 | 0.46 | 1.25 |
| 1280x720 | lanes | yes | 0.58 | 0.40 | 1.56 |
| 1920x1080 | now | yes | 13.29 | 4.51 | 17.06 |
| 1920x1080 | pairs | yes | 1.72 | 0.89 | 3.05 |
| 1920x1080 | lanes | yes | 1.30 | 0.70 | 1.99 |
| 3840x2160 | now | yes | 52.98 | 16.88 | 66.49 |
| 3840x2160 | pairs | yes | 6.74 | 3.24 | 11.03 |
| 3840x2160 | lanes | yes | 5.04 | 2.49 | 8.24 |

