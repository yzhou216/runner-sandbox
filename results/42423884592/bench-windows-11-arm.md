# windows-11-arm

Cobalt 100
4

## Kernels of the conversion, 4 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 0.52 | 0.17 | 0.51 |
| 640x360 | pairs | yes | 0.27 | 0.10 | 0.31 |
| 640x360 | lanes | yes | 0.25 | 0.10 | 0.47 |
| 1280x720 | now | yes | 2.67 | 1.05 | 2.97 |
| 1280x720 | pairs | yes | 1.67 | 0.86 | 2.85 |
| 1280x720 | lanes | yes | 1.56 | 0.84 | 2.03 |
| 1920x1080 | now | yes | 5.96 | 2.14 | 6.24 |
| 1920x1080 | pairs | yes | 3.71 | 1.58 | 4.57 |
| 1920x1080 | lanes | yes | 3.44 | 1.51 | 3.75 |
| 3840x2160 | now | yes | 23.10 | 7.11 | 24.69 |
| 3840x2160 | pairs | yes | 13.98 | 4.89 | 15.05 |
| 3840x2160 | lanes | yes | 12.86 | 4.62 | 14.00 |

## Passes through whole clips, 4 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p-noise | now, now | 120 | 15.7 | 1.05 |
| 1080p-noise | pipelined, now | 120 | 15.8 | 1.07 |
| 1080p-noise | threaded, now | 120 | 46.9 | 3.25 |
| 1080p-noise | now, lanes | 120 | 15.8 | 1.05 |
| 1080p-noise | pipelined, lanes | 120 | 15.9 | 1.05 |
| 1080p-noise | threaded, lanes | 120 | 49.0 | 3.23 |
| 1080p | now, now | 180 | 75.4 | 1.28 |
| 1080p | pipelined, now | 180 | 78.2 | 1.45 |
| 1080p | threaded, now | 180 | 187.2 | 3.36 |
| 1080p | now, lanes | 180 | 75.3 | 1.17 |
| 1080p | pipelined, lanes | 180 | 81.9 | 1.32 |
| 1080p | threaded, lanes | 180 | 212.8 | 3.20 |
| 2160p-noise | now, now | 60 | 4.3 | 1.08 |
| 2160p-noise | pipelined, now | 60 | 4.3 | 1.09 |
| 2160p-noise | threaded, now | 60 | 13.2 | 3.44 |
| 2160p-noise | now, lanes | 60 | 4.3 | 1.05 |
| 2160p-noise | pipelined, lanes | 60 | 4.4 | 1.05 |
| 2160p-noise | threaded, lanes | 60 | 13.7 | 3.42 |
| 2160p | now, now | 120 | 18.7 | 1.31 |
| 2160p | pipelined, now | 120 | 19.1 | 1.32 |
| 2160p | threaded, now | 120 | 47.1 | 3.33 |
| 2160p | now, lanes | 120 | 19.7 | 1.22 |
| 2160p | pipelined, lanes | 120 | 20.4 | 1.20 |
| 2160p | threaded, lanes | 120 | 51.8 | 3.15 |
| 720p-noise | now, now | 180 | 37.2 | 1.09 |
| 720p-noise | pipelined, now | 180 | 37.7 | 1.05 |
| 720p-noise | threaded, now | 180 | 97.2 | 2.84 |
| 720p-noise | now, lanes | 180 | 37.6 | 1.05 |
| 720p-noise | pipelined, lanes | 180 | 38.2 | 1.05 |
| 720p-noise | threaded, lanes | 180 | 99.8 | 2.85 |

