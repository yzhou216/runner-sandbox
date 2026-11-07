# windows-latest

AMD EPYC 9V45 96-Core Processor                
4

## Kernels of the conversion, 4 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 1.61 | 0.53 | 1.95 |
| 640x360 | pairs | yes | 0.38 | 0.26 | 0.94 |
| 640x360 | lanes | yes | 0.40 | 0.24 | 0.86 |
| 1280x720 | now | yes | 6.69 | 2.24 | 8.60 |
| 1280x720 | pairs | yes | 1.85 | 0.89 | 3.55 |
| 1280x720 | lanes | yes | 1.75 | 0.83 | 3.40 |
| 1920x1080 | now | yes | 15.05 | 4.76 | 18.56 |
| 1920x1080 | pairs | yes | 4.00 | 1.88 | 6.99 |
| 1920x1080 | lanes | yes | 3.89 | 1.69 | 6.37 |
| 3840x2160 | now | yes | 61.60 | 19.42 | 73.17 |
| 3840x2160 | pairs | yes | 16.62 | 7.54 | 27.08 |
| 3840x2160 | lanes | yes | 15.44 | 6.41 | 22.61 |

## cinq's conversion of high definition, at ec61ead and in the snapshot

| size | ec61ead ms | ec61ead CPU ms | snapshot ms | snapshot CPU ms | same bytes |
| --- | --- | --- | --- | --- | --- |
| 1280x720 | 2.33 | 9.29 | 1.17 | 4.10 | yes |
| 1920x1080 | 4.99 | 18.47 | 2.32 | 8.20 | yes |
| 3840x2160 | 19.33 | 74.31 | 9.08 | 33.23 | yes |

