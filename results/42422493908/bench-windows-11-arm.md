# windows-11-arm

Cobalt 100
4

## Conversion to RGBA, 4 strips

| size | way | burst ms | burst CPU ms | paced median ms | paced p95 ms | paced CPU ms |
| --- | --- | --- | --- | --- | --- | --- |
| 640x360 | serial | 0.67 | 0.66 | 0.70 | 0.73 | 1.04 |
| 640x360 | scoped | 0.55 | 1.52 | 0.61 | 0.65 | 0.35 |
| 640x360 | rayon-strips | 0.40 | 0.90 | 0.44 | 0.49 | 0.17 |
| 640x360 | rayon-rows | 0.42 | 1.37 | 0.45 | 0.52 | 3.99 |
| 640x360 | rayon-strips, buffer reused | 0.14 | 0.62 | | | |
| 1280x720 | serial | 2.68 | 2.67 | 2.72 | 2.77 | 2.95 |
| 1280x720 | scoped | 1.23 | 3.79 | 1.31 | 1.36 | 3.82 |
| 1280x720 | rayon-strips | 1.04 | 2.93 | 1.10 | 1.18 | 4.17 |
| 1280x720 | rayon-rows | 1.13 | 3.55 | 1.20 | 1.28 | 4.17 |
| 1280x720 | rayon-strips, buffer reused | 0.54 | 2.30 | | | |
| 1920x1080 | serial | 6.02 | 5.99 | 6.09 | 6.16 | 5.21 |
| 1920x1080 | scoped | 2.34 | 6.92 | 2.41 | 2.49 | 6.08 |
| 1920x1080 | rayon-strips | 2.16 | 6.84 | 2.23 | 2.28 | 5.73 |
| 1920x1080 | rayon-rows | 2.27 | 5.87 | 2.31 | 2.46 | 8.51 |
| 1920x1080 | rayon-strips, buffer reused | 1.19 | 4.65 | | | |
| 3840x2160 | serial | 22.95 | 22.77 | 22.88 | 23.30 | 22.92 |
| 3840x2160 | scoped | 7.27 | 24.94 | 7.29 | 8.66 | 24.83 |
| 3840x2160 | rayon-strips | 7.04 | 24.12 | 7.07 | 7.34 | 25.00 |
| 3840x2160 | rayon-rows | 7.15 | 24.14 | 7.26 | 7.68 | 23.26 |
| 3840x2160 | rayon-strips, buffer reused | 4.68 | 18.66 | | | |

## Drawing a new picture with tiny-skia, on the thread of the interface

| picture | screen | now ms | scale ms, on rayon | prescaled ms |
| --- | --- | --- | --- | --- |
| 640x360 | 1280x720 | 15.91 | 5.33 | 7.92 |
| 640x360 | 1920x1080 | 35.24 | 11.73 | 17.73 |
| 1280x720 | 1280x720 | 7.85 | 5.32 | 7.21 |
| 1280x720 | 1920x1080 | 36.49 | 11.81 | 17.76 |
| 1920x1080 | 1280x720 | 21.04 | 4.91 | 7.21 |
| 1920x1080 | 1920x1080 | 17.78 | 11.76 | 16.19 |
| 3840x2160 | 1280x720 | 35.39 | 5.86 | 7.87 |
| 3840x2160 | 1920x1080 | 54.93 | 11.81 | 17.67 |

## Passes through whole clips, 4 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p-noise | now | 120 | 15.7 | 1.08 |
| 1080p-noise | rayon | 120 | 15.8 | 1.05 |
| 1080p-noise | pipelined | 120 | 15.9 | 1.06 |
| 1080p-noise | threaded | 120 | 47.9 | 3.28 |
| 1080p | now | 180 | 75.5 | 1.46 |
| 1080p | rayon | 180 | 76.8 | 1.46 |
| 1080p | pipelined | 180 | 78.6 | 1.38 |
| 1080p | threaded | 180 | 196.1 | 3.40 |
| 2160p-noise | now | 60 | 4.3 | 1.06 |
| 2160p-noise | rayon | 60 | 4.3 | 1.08 |
| 2160p-noise | pipelined | 60 | 4.3 | 1.07 |
| 2160p-noise | threaded | 60 | 13.2 | 3.39 |
| 2160p | now | 120 | 19.0 | 1.33 |
| 2160p | rayon | 120 | 19.0 | 1.33 |
| 2160p | pipelined | 120 | 19.5 | 1.37 |
| 2160p | threaded | 120 | 47.9 | 3.36 |
| 720p-noise | now | 180 | 37.3 | 1.09 |
| 720p-noise | rayon | 180 | 37.6 | 1.07 |
| 720p-noise | pipelined | 180 | 37.9 | 1.04 |
| 720p-noise | threaded | 180 | 97.9 | 2.89 |

