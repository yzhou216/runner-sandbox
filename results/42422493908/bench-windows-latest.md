# windows-latest

AMD EPYC 7763 64-Core Processor                
4

## Conversion to RGBA, 4 strips

| size | way | burst ms | burst CPU ms | paced median ms | paced p95 ms | paced CPU ms |
| --- | --- | --- | --- | --- | --- | --- |
| 640x360 | serial | 2.09 | 2.09 | 2.14 | 2.32 | 2.43 |
| 640x360 | scoped | 1.06 | 3.44 | 1.09 | 1.46 | 3.30 |
| 640x360 | rayon-strips | 0.78 | 2.38 | 0.84 | 1.40 | 3.30 |
| 640x360 | rayon-rows | 0.93 | 3.20 | 0.97 | 1.15 | 3.65 |
| 640x360 | rayon-strips, buffer reused | 0.71 | 2.38 | | | |
| 1280x720 | serial | 8.79 | 8.59 | 8.86 | 9.07 | 9.03 |
| 1280x720 | scoped | 3.27 | 11.96 | 3.35 | 5.78 | 12.33 |
| 1280x720 | rayon-strips | 3.01 | 11.11 | 3.10 | 5.22 | 11.81 |
| 1280x720 | rayon-rows | 3.20 | 11.27 | 3.25 | 3.97 | 12.67 |
| 1280x720 | rayon-strips, buffer reused | 2.71 | 10.60 | | | |
| 1920x1080 | serial | 19.73 | 19.82 | 19.76 | 20.03 | 19.62 |
| 1920x1080 | scoped | 6.79 | 24.46 | 6.90 | 11.64 | 26.04 |
| 1920x1080 | rayon-strips | 6.51 | 25.78 | 6.64 | 11.49 | 24.65 |
| 1920x1080 | rayon-rows | 6.67 | 25.21 | 6.75 | 8.77 | 28.12 |
| 1920x1080 | rayon-strips, buffer reused | 5.95 | 23.18 | | | |
| 3840x2160 | serial | 78.77 | 78.12 | 78.91 | 81.34 | 78.82 |
| 3840x2160 | scoped | 25.75 | 96.88 | 25.77 | 34.97 | 98.26 |
| 3840x2160 | rayon-strips | 25.48 | 96.35 | 25.58 | 37.43 | 97.22 |
| 3840x2160 | rayon-rows | 25.88 | 96.15 | 25.92 | 30.83 | 97.22 |
| 3840x2160 | rayon-strips, buffer reused | 23.75 | 90.82 | | | |

## Drawing a new picture with tiny-skia, on the thread of the interface

| picture | screen | now ms | scale ms, on rayon | prescaled ms |
| --- | --- | --- | --- | --- |
| 640x360 | 1280x720 | 14.13 | 8.91 | 7.29 |
| 640x360 | 1920x1080 | 31.58 | 19.79 | 16.45 |
| 1280x720 | 1280x720 | 7.32 | 8.95 | 7.33 |
| 1280x720 | 1920x1080 | 32.94 | 19.77 | 16.46 |
| 1920x1080 | 1280x720 | 17.34 | 8.97 | 7.27 |
| 1920x1080 | 1920x1080 | 16.45 | 19.75 | 16.44 |
| 3840x2160 | 1280x720 | 27.20 | 9.03 | 7.32 |
| 3840x2160 | 1920x1080 | 44.65 | 19.87 | 16.45 |

## Passes through whole clips, 4 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p-noise | now | 120 | 10.2 | 1.13 |
| 1080p-noise | rayon | 120 | 10.3 | 1.12 |
| 1080p-noise | pipelined | 120 | 10.5 | 1.18 |
| 1080p-noise | threaded | 120 | 24.2 | 3.39 |
| 1080p | now | 180 | 51.4 | 1.65 |
| 1080p | rayon | 180 | 51.8 | 1.46 |
| 1080p | pipelined | 180 | 56.0 | 1.78 |
| 1080p | threaded | 180 | 93.7 | 3.57 |
| 2160p-noise | now | 60 | 2.6 | 1.12 |
| 2160p-noise | rayon | 60 | 2.7 | 1.13 |
| 2160p-noise | pipelined | 60 | 2.7 | 1.16 |
| 2160p-noise | threaded | 60 | 6.3 | 3.45 |
| 2160p | now | 120 | 12.9 | 1.53 |
| 2160p | rayon | 120 | 12.9 | 1.53 |
| 2160p | pipelined | 120 | 13.6 | 1.64 |
| 2160p | threaded | 120 | 23.1 | 3.42 |
| 720p-noise | now | 180 | 24.5 | 1.16 |
| 720p-noise | rayon | 180 | 24.6 | 1.15 |
| 720p-noise | pipelined | 180 | 25.2 | 1.18 |
| 720p-noise | threaded | 180 | 52.1 | 2.90 |

