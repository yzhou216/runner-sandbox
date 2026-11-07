# ubuntu-24.04-riscv

CPU(s):              4

## Conversion to RGBA, 4 strips

| size | way | burst ms | burst CPU ms | paced median ms | paced p95 ms | paced CPU ms |
| --- | --- | --- | --- | --- | --- | --- |
| 640x360 | serial | 4.76 | 4.77 | 4.81 | 6.23 | 4.97 |
| 640x360 | scoped | 2.90 | 2.49 | 3.63 | 8.56 | 7.53 |
| 640x360 | rayon-strips | 1.30 | 5.24 | 4.05 | 8.24 | 15.01 |
| 640x360 | rayon-rows | 1.39 | 5.51 | 4.49 | 8.44 | 17.32 |
| 640x360 | rayon-strips, buffer reused | 1.21 | 5.13 | | | |
| 1280x720 | serial | 18.87 | 19.02 | 18.85 | 19.62 | 18.89 |
| 1280x720 | scoped | 10.29 | 14.63 | 10.09 | 18.29 | 22.75 |
| 1280x720 | rayon-strips | 5.11 | 19.87 | 9.88 | 19.62 | 35.00 |
| 1280x720 | rayon-rows | 5.51 | 20.77 | 9.33 | 22.12 | 38.84 |
| 1280x720 | rayon-strips, buffer reused | 4.79 | 19.28 | | | |
| 1920x1080 | serial | 42.70 | 43.01 | 42.61 | 42.83 | 42.58 |
| 1920x1080 | scoped | 11.86 | 37.33 | 14.04 | 19.38 | 44.96 |
| 1920x1080 | rayon-strips | 11.54 | 43.63 | 14.19 | 17.82 | 52.07 |
| 1920x1080 | rayon-rows | 11.65 | 44.05 | 14.70 | 20.32 | 51.30 |
| 1920x1080 | rayon-strips, buffer reused | 21.73 | 43.19 | | | |
| 3840x2160 | serial | 172.11 | 171.26 | 170.04 | 171.53 | 169.93 |
| 3840x2160 | scoped | 50.18 | 169.36 | 47.79 | 79.58 | 166.62 |
| 3840x2160 | rayon-strips | 46.63 | 174.86 | 46.48 | 63.39 | 174.61 |
| 3840x2160 | rayon-rows | 46.29 | 174.70 | 53.14 | 72.98 | 174.29 |
| 3840x2160 | rayon-strips, buffer reused | 42.93 | 168.46 | | | |

## Drawing a new picture with tiny-skia, on the thread of the interface

| picture | screen | now ms | scale ms, on rayon | prescaled ms |
| --- | --- | --- | --- | --- |
| 640x360 | 1280x720 | 199.37 | 18.31 | 106.38 |
| 640x360 | 1920x1080 | 454.13 | 58.45 | 243.68 |
| 1280x720 | 1280x720 | 112.07 | 18.06 | 113.15 |
| 1280x720 | 1920x1080 | 505.97 | 41.21 | 242.20 |
| 1920x1080 | 1280x720 | 221.85 | 29.16 | 107.31 |
| 1920x1080 | 1920x1080 | 251.02 | 40.52 | 253.29 |
| 3840x2160 | 1280x720 | 279.64 | 18.24 | 111.87 |
| 3840x2160 | 1920x1080 | 560.72 | 41.47 | 243.96 |

## Passes through whole clips, 4 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 720p-noise | now | 180 | 5.3 | 1.03 |
| 720p-noise | rayon | 180 | 5.2 | 1.04 |
| 720p-noise | pipelined | 180 | 5.6 | 1.09 |
| 720p-noise | threaded | 180 | 13.6 | 2.69 |
| 1080p | now | 180 | 8.1 | 1.12 |
| 1080p | rayon | 180 | 8.0 | 1.17 |
| 1080p | pipelined | 180 | 9.2 | 1.31 |
| 1080p | threaded | 180 | 18.7 | 2.90 |

