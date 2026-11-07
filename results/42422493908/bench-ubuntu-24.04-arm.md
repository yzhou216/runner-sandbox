# ubuntu-24.04-arm

CPU(s):                                  4
Model name:                              Neoverse-N2

## Conversion to RGBA, 4 strips

| size | way | burst ms | burst CPU ms | paced median ms | paced p95 ms | paced CPU ms |
| --- | --- | --- | --- | --- | --- | --- |
| 640x360 | serial | 0.53 | 0.53 | 0.53 | 0.55 | 0.54 |
| 640x360 | scoped | 0.25 | 0.78 | 0.28 | 0.32 | 0.87 |
| 640x360 | rayon-strips | 0.16 | 0.62 | 0.20 | 0.32 | 0.71 |
| 640x360 | rayon-rows | 0.18 | 0.66 | 0.21 | 0.35 | 0.72 |
| 640x360 | rayon-strips, buffer reused | 0.13 | 0.54 | | | |
| 1280x720 | serial | 2.14 | 2.14 | 2.15 | 2.17 | 2.16 |
| 1280x720 | scoped | 0.73 | 2.47 | 0.75 | 0.78 | 2.55 |
| 1280x720 | rayon-strips | 0.61 | 2.25 | 0.65 | 0.86 | 2.34 |
| 1280x720 | rayon-rows | 0.70 | 2.39 | 0.73 | 0.83 | 2.48 |
| 1280x720 | rayon-strips, buffer reused | 0.53 | 2.14 | | | |
| 1920x1080 | serial | 4.81 | 4.81 | 4.84 | 4.88 | 4.86 |
| 1920x1080 | scoped | 1.47 | 5.20 | 1.48 | 2.70 | 5.27 |
| 1920x1080 | rayon-strips | 1.35 | 4.94 | 1.40 | 1.62 | 5.04 |
| 1920x1080 | rayon-rows | 1.40 | 5.05 | 1.46 | 1.56 | 5.16 |
| 1920x1080 | rayon-strips, buffer reused | 1.19 | 4.75 | | | |
| 3840x2160 | serial | 19.30 | 19.31 | 19.33 | 19.45 | 19.32 |
| 3840x2160 | scoped | 5.44 | 19.74 | 5.49 | 5.66 | 19.91 |
| 3840x2160 | rayon-strips | 5.30 | 19.39 | 5.36 | 5.52 | 19.52 |
| 3840x2160 | rayon-rows | 5.37 | 19.59 | 5.42 | 5.56 | 19.66 |
| 3840x2160 | rayon-strips, buffer reused | 4.73 | 18.82 | | | |

## Drawing a new picture with tiny-skia, on the thread of the interface

| picture | screen | now ms | scale ms, on rayon | prescaled ms |
| --- | --- | --- | --- | --- |
| 640x360 | 1280x720 | 15.84 | 4.84 | 7.24 |
| 640x360 | 1920x1080 | 35.09 | 10.88 | 16.27 |
| 1280x720 | 1280x720 | 7.25 | 4.87 | 7.23 |
| 1280x720 | 1920x1080 | 36.54 | 10.84 | 16.26 |
| 1920x1080 | 1280x720 | 19.28 | 4.85 | 7.22 |
| 1920x1080 | 1920x1080 | 16.27 | 10.88 | 16.26 |
| 3840x2160 | 1280x720 | 30.83 | 4.86 | 7.22 |
| 3840x2160 | 1920x1080 | 50.16 | 10.91 | 16.26 |

## Passes through whole clips, 4 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p-noise | now | 120 | 15.6 | 1.05 |
| 1080p-noise | rayon | 120 | 15.8 | 1.06 |
| 1080p-noise | pipelined | 120 | 16.2 | 1.08 |
| 1080p-noise | threaded | 120 | 47.3 | 3.23 |
| 1080p | now | 180 | 78.2 | 1.25 |
| 1080p | rayon | 180 | 82.7 | 1.30 |
| 1080p | pipelined | 180 | 94.2 | 1.48 |
| 1080p | threaded | 180 | 197.9 | 3.34 |
| 2160p-noise | now | 60 | 4.4 | 1.06 |
| 2160p-noise | rayon | 60 | 4.4 | 1.06 |
| 2160p-noise | pipelined | 60 | 4.5 | 1.09 |
| 2160p-noise | threaded | 60 | 14.0 | 3.42 |
| 2160p | now | 120 | 20.7 | 1.29 |
| 2160p | rayon | 120 | 20.9 | 1.30 |
| 2160p | pipelined | 120 | 23.8 | 1.47 |
| 2160p | threaded | 120 | 51.6 | 3.36 |
| 720p-noise | now | 180 | 37.0 | 1.06 |
| 720p-noise | rayon | 180 | 37.6 | 1.06 |
| 720p-noise | pipelined | 180 | 38.7 | 1.09 |
| 720p-noise | threaded | 180 | 99.7 | 2.88 |

