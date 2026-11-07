# ubuntu-latest

CPU(s):                                  4
Model name:                              AMD EPYC 7763 64-Core Processor

## Conversion to RGBA, 4 strips

| size | way | burst ms | burst CPU ms | paced median ms | paced p95 ms | paced CPU ms |
| --- | --- | --- | --- | --- | --- | --- |
| 640x360 | serial | 2.05 | 2.06 | 2.34 | 2.55 | 2.35 |
| 640x360 | scoped | 0.79 | 2.89 | 0.97 | 1.07 | 3.56 |
| 640x360 | rayon-strips | 0.69 | 2.74 | 0.96 | 1.11 | 3.61 |
| 640x360 | rayon-rows | 0.74 | 2.83 | 1.01 | 1.18 | 3.84 |
| 640x360 | rayon-strips, buffer reused | 0.66 | 2.64 | | | |
| 1280x720 | serial | 8.26 | 8.28 | 8.47 | 8.60 | 8.48 |
| 1280x720 | scoped | 2.82 | 10.82 | 3.10 | 3.34 | 11.60 |
| 1280x720 | rayon-strips | 2.73 | 10.64 | 3.01 | 4.97 | 11.48 |
| 1280x720 | rayon-rows | 2.86 | 10.73 | 2.97 | 3.10 | 11.38 |
| 1280x720 | rayon-strips, buffer reused | 2.63 | 10.47 | | | |
| 1920x1080 | serial | 18.97 | 19.04 | 18.65 | 18.86 | 18.68 |
| 1920x1080 | scoped | 6.16 | 23.89 | 6.27 | 6.45 | 24.13 |
| 1920x1080 | rayon-strips | 6.13 | 23.70 | 6.33 | 6.50 | 24.39 |
| 1920x1080 | rayon-rows | 6.19 | 23.87 | 6.47 | 7.00 | 24.60 |
| 1920x1080 | rayon-strips, buffer reused | 5.88 | 23.50 | | | |
| 3840x2160 | serial | 75.56 | 75.58 | 75.54 | 76.14 | 75.77 |
| 3840x2160 | scoped | 25.52 | 96.00 | 25.41 | 25.78 | 95.90 |
| 3840x2160 | rayon-strips | 25.53 | 95.65 | 25.44 | 29.66 | 95.45 |
| 3840x2160 | rayon-rows | 25.25 | 95.21 | 25.30 | 26.04 | 95.34 |
| 3840x2160 | rayon-strips, buffer reused | 23.53 | 93.70 | | | |

## Drawing a new picture with tiny-skia, on the thread of the interface

| picture | screen | now ms | scale ms, on rayon | prescaled ms |
| --- | --- | --- | --- | --- |
| 640x360 | 1280x720 | 13.50 | 8.56 | 6.50 |
| 640x360 | 1920x1080 | 30.15 | 19.26 | 14.77 |
| 1280x720 | 1280x720 | 6.62 | 8.61 | 6.52 |
| 1280x720 | 1920x1080 | 31.17 | 19.44 | 14.79 |
| 1920x1080 | 1280x720 | 15.66 | 8.67 | 6.57 |
| 1920x1080 | 1920x1080 | 15.00 | 19.45 | 14.80 |
| 3840x2160 | 1280x720 | 21.56 | 8.68 | 6.54 |
| 3840x2160 | 1920x1080 | 38.25 | 19.60 | 14.76 |

## Passes through whole clips, 4 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p-noise | now | 120 | 10.8 | 1.13 |
| 1080p-noise | rayon | 120 | 10.8 | 1.14 |
| 1080p-noise | pipelined | 120 | 11.2 | 1.20 |
| 1080p-noise | threaded | 120 | 25.8 | 3.43 |
| 1080p | now | 180 | 56.4 | 1.55 |
| 1080p | rayon | 180 | 56.8 | 1.56 |
| 1080p | pipelined | 180 | 64.7 | 1.82 |
| 1080p | threaded | 180 | 101.9 | 3.52 |
| 2160p-noise | now | 60 | 2.9 | 1.14 |
| 2160p-noise | rayon | 60 | 2.9 | 1.14 |
| 2160p-noise | pipelined | 60 | 3.0 | 1.20 |
| 2160p-noise | threaded | 60 | 6.9 | 3.49 |
| 2160p | now | 120 | 14.0 | 1.53 |
| 2160p | rayon | 120 | 14.3 | 1.54 |
| 2160p | pipelined | 120 | 16.4 | 1.80 |
| 2160p | threaded | 120 | 26.8 | 3.59 |
| 720p-noise | now | 180 | 25.3 | 1.14 |
| 720p-noise | rayon | 180 | 25.4 | 1.16 |
| 720p-noise | pipelined | 180 | 26.3 | 1.23 |
| 720p-noise | threaded | 180 | 54.3 | 2.97 |

