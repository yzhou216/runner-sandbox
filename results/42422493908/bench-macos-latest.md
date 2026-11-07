# macos-latest

Apple M1 (Virtual)
3

## Conversion to RGBA, 3 strips

| size | way | burst ms | burst CPU ms | paced median ms | paced p95 ms | paced CPU ms |
| --- | --- | --- | --- | --- | --- | --- |
| 640x360 | serial | 0.59 | 0.65 | 0.62 | 1.73 | 0.86 |
| 640x360 | scoped | 0.41 | 0.91 | 0.46 | 1.21 | 1.06 |
| 640x360 | rayon-strips | 0.39 | 0.83 | 0.63 | 1.27 | 1.39 |
| 640x360 | rayon-rows | 0.30 | 0.81 | 0.51 | 1.26 | 1.20 |
| 640x360 | rayon-strips, buffer reused | 0.58 | 1.11 | | | |
| 1280x720 | serial | 2.48 | 3.04 | 3.20 | 6.05 | 3.54 |
| 1280x720 | scoped | 1.01 | 2.85 | 2.23 | 4.42 | 3.83 |
| 1280x720 | rayon-strips | 3.66 | 3.94 | 5.07 | 21.84 | 4.55 |
| 1280x720 | rayon-rows | 1.81 | 3.71 | 3.76 | 36.18 | 4.55 |
| 1280x720 | rayon-strips, buffer reused | 4.05 | 4.50 | | | |
| 1920x1080 | serial | 7.12 | 7.20 | 7.67 | 14.34 | 8.26 |
| 1920x1080 | scoped | 4.94 | 7.90 | 5.00 | 12.88 | 8.78 |
| 1920x1080 | rayon-strips | 2.71 | 7.16 | 4.80 | 20.72 | 8.19 |
| 1920x1080 | rayon-rows | 2.85 | 6.88 | 3.30 | 40.54 | 6.93 |
| 1920x1080 | rayon-strips, buffer reused | 4.17 | 7.68 | | | |
| 3840x2160 | serial | 29.49 | 29.93 | 26.67 | 35.66 | 27.17 |
| 3840x2160 | scoped | 14.58 | 32.73 | 14.88 | 28.09 | 30.43 |
| 3840x2160 | rayon-strips | 30.00 | 28.04 | 24.89 | 40.50 | 30.65 |
| 3840x2160 | rayon-rows | 11.67 | 24.01 | 12.56 | 50.22 | 29.12 |
| 3840x2160 | rayon-strips, buffer reused | 28.36 | 27.26 | | | |

## Drawing a new picture with tiny-skia, on the thread of the interface

| picture | screen | now ms | scale ms, on rayon | prescaled ms |
| --- | --- | --- | --- | --- |
| 640x360 | 1280x720 | 19.26 | 6.07 | 10.30 |
| 640x360 | 1920x1080 | 41.23 | 17.21 | 28.77 |
| 1280x720 | 1280x720 | 12.03 | 4.22 | 9.99 |
| 1280x720 | 1920x1080 | 35.67 | 10.02 | 22.95 |
| 1920x1080 | 1280x720 | 28.00 | 9.50 | 10.91 |
| 1920x1080 | 1920x1080 | 28.78 | 12.30 | 24.88 |
| 3840x2160 | 1280x720 | 41.17 | 6.33 | 12.32 |
| 3840x2160 | 1920x1080 | 71.63 | 15.73 | 25.80 |

## Passes through whole clips, 3 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p-noise | now | 120 | 10.5 | 0.99 |
| 1080p-noise | rayon | 120 | 12.4 | 1.01 |
| 1080p-noise | pipelined | 120 | 13.6 | 1.07 |
| 1080p-noise | threaded | 120 | 21.3 | 2.05 |
| 1080p | now | 180 | 50.0 | 1.10 |
| 1080p | rayon | 180 | 55.2 | 1.16 |
| 1080p | pipelined | 180 | 98.4 | 1.62 |
| 1080p | threaded | 180 | 157.4 | 2.76 |
| 2160p-noise | now | 60 | 4.1 | 1.05 |
| 2160p-noise | rayon | 60 | 3.8 | 1.05 |
| 2160p-noise | pipelined | 60 | 4.1 | 1.09 |
| 2160p-noise | threaded | 60 | 8.8 | 2.52 |
| 2160p | now | 120 | 24.4 | 1.32 |
| 2160p | rayon | 120 | 21.8 | 1.34 |
| 2160p | pipelined | 120 | 25.7 | 1.51 |
| 2160p | threaded | 120 | 42.0 | 2.63 |
| 720p-noise | now | 180 | 34.8 | 1.06 |
| 720p-noise | rayon | 180 | 32.7 | 1.06 |
| 720p-noise | pipelined | 180 | 38.6 | 1.11 |
| 720p-noise | threaded | 180 | 70.5 | 2.14 |

