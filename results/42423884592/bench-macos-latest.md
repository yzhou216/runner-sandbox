# macos-latest

Apple M1 (Virtual)
3

## Conversion to RGBA, 3 strips

| size | way | burst ms | burst CPU ms | paced median ms | paced p95 ms | paced CPU ms |
| --- | --- | --- | --- | --- | --- | --- |
| 640x360 | serial | 0.54 | 0.69 | 0.64 | 1.76 | 0.87 |
| 640x360 | scoped | 0.65 | 0.98 | 0.71 | 1.66 | 1.24 |
| 640x360 | rayon-strips | 0.59 | 0.93 | 0.67 | 15.81 | 1.15 |
| 640x360 | rayon-rows | 0.50 | 1.02 | 2.20 | 20.39 | 1.14 |
| 640x360 | rayon-strips, buffer reused | 0.63 | 1.02 | | | |
| 1280x720 | serial | 2.90 | 3.42 | 2.98 | 6.85 | 3.63 |
| 1280x720 | scoped | 1.60 | 3.34 | 2.40 | 2.72 | 4.04 |
| 1280x720 | rayon-strips | 2.14 | 3.54 | 2.37 | 7.42 | 3.97 |
| 1280x720 | rayon-rows | 1.68 | 3.21 | 3.53 | 30.98 | 3.69 |
| 1280x720 | rayon-strips, buffer reused | 2.71 | 3.11 | | | |
| 1920x1080 | serial | 5.24 | 6.46 | 5.12 | 11.73 | 5.92 |
| 1920x1080 | scoped | 3.81 | 7.46 | 4.72 | 12.38 | 9.00 |
| 1920x1080 | rayon-strips | 3.73 | 7.98 | 3.93 | 10.25 | 8.62 |
| 1920x1080 | rayon-rows | 3.05 | 7.79 | 3.70 | 10.98 | 9.93 |
| 1920x1080 | rayon-strips, buffer reused | 4.74 | 7.38 | | | |
| 3840x2160 | serial | 26.41 | 25.95 | 26.44 | 39.30 | 26.34 |
| 3840x2160 | scoped | 13.96 | 31.12 | 13.96 | 33.15 | 28.67 |
| 3840x2160 | rayon-strips | 13.77 | 28.29 | 13.14 | 20.95 | 27.24 |
| 3840x2160 | rayon-rows | 9.14 | 25.75 | 9.76 | 13.97 | 26.45 |
| 3840x2160 | rayon-strips, buffer reused | 13.44 | 26.08 | | | |

## Kernels of the conversion, 3 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 0.55 | 0.47 | 1.09 |
| 640x360 | pairs | yes | 0.14 | 0.20 | 0.38 |
| 640x360 | lanes | yes | 0.12 | 0.17 | 0.39 |
| 1280x720 | now | yes | 2.13 | 2.33 | 3.63 |
| 1280x720 | pairs | yes | 0.57 | 0.59 | 1.00 |
| 1280x720 | lanes | yes | 0.53 | 0.58 | 0.92 |
| 1920x1080 | now | yes | 5.40 | 3.64 | 8.04 |
| 1920x1080 | pairs | yes | 1.26 | 1.26 | 2.01 |
| 1920x1080 | lanes | yes | 1.15 | 1.01 | 2.10 |
| 3840x2160 | now | yes | 22.86 | 14.32 | 29.80 |
| 3840x2160 | pairs | yes | 5.67 | 5.60 | 8.41 |
| 3840x2160 | lanes | yes | 5.19 | 4.29 | 7.61 |

## Passes through whole clips, 3 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p-noise | now, now | 120 | 11.6 | 1.03 |
| 1080p-noise | pipelined, now | 120 | 12.3 | 1.08 |
| 1080p-noise | threaded, now | 120 | 24.6 | 2.29 |
| 1080p-noise | now, lanes | 120 | 13.0 | 1.01 |
| 1080p-noise | pipelined, lanes | 120 | 11.9 | 0.99 |
| 1080p-noise | threaded, lanes | 120 | 22.6 | 1.96 |
| 1080p | now, now | 180 | 53.8 | 1.18 |
| 1080p | pipelined, now | 180 | 87.3 | 1.81 |
| 1080p | threaded, now | 180 | 137.7 | 2.78 |
| 1080p | now, lanes | 180 | 93.8 | 1.10 |
| 1080p | pipelined, lanes | 180 | 102.9 | 1.35 |
| 1080p | threaded, lanes | 180 | 145.8 | 2.26 |
| 2160p-noise | now, now | 60 | 3.5 | 1.05 |
| 2160p-noise | pipelined, now | 60 | 4.0 | 1.08 |
| 2160p-noise | threaded, now | 60 | 9.7 | 2.57 |
| 2160p-noise | now, lanes | 60 | 4.2 | 1.01 |
| 2160p-noise | pipelined, lanes | 60 | 4.0 | 1.03 |
| 2160p-noise | threaded, lanes | 60 | 10.3 | 2.55 |
| 2160p | now, now | 120 | 15.8 | 1.26 |
| 2160p | pipelined, now | 120 | 20.0 | 1.52 |
| 2160p | threaded, now | 120 | 33.1 | 2.48 |
| 2160p | now, lanes | 120 | 25.2 | 1.09 |
| 2160p | pipelined, lanes | 120 | 25.9 | 1.19 |
| 2160p | threaded, lanes | 120 | 50.8 | 2.24 |
| 720p-noise | now, now | 180 | 38.5 | 1.05 |
| 720p-noise | pipelined, now | 180 | 36.3 | 1.14 |
| 720p-noise | threaded, now | 180 | 62.2 | 2.25 |
| 720p-noise | now, lanes | 180 | 36.4 | 1.02 |
| 720p-noise | pipelined, lanes | 180 | 37.8 | 1.05 |
| 720p-noise | threaded, lanes | 180 | 67.7 | 2.03 |

