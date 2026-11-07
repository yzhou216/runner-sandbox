# windows-latest

AMD EPYC 9V45 96-Core Processor                
4

## Kernels of the conversion, 4 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 1.62 | 0.53 | 2.15 |
| 640x360 | pairs | yes | 0.38 | 0.21 | 0.98 |
| 640x360 | lanes | yes | 0.40 | 0.23 | 0.62 |
| 1280x720 | now | yes | 6.55 | 2.12 | 7.94 |
| 1280x720 | pairs | yes | 1.79 | 0.88 | 3.12 |
| 1280x720 | lanes | yes | 1.75 | 0.79 | 2.70 |
| 1920x1080 | now | yes | 14.72 | 4.93 | 18.63 |
| 1920x1080 | pairs | yes | 4.15 | 1.86 | 6.88 |
| 1920x1080 | lanes | yes | 3.90 | 1.69 | 6.05 |
| 3840x2160 | now | yes | 62.26 | 19.90 | 75.39 |
| 3840x2160 | pairs | yes | 15.98 | 7.29 | 26.83 |
| 3840x2160 | lanes | yes | 15.42 | 6.45 | 23.96 |

## Passes through whole clips, 4 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p-noise | now, now | 120 | 15.1 | 1.09 |
| 1080p-noise | pipelined, now | 120 | 15.7 | 1.11 |
| 1080p-noise | threaded, now | 120 | 38.0 | 3.36 |
| 1080p-noise | now, lanes | 120 | 15.9 | 1.09 |
| 1080p-noise | pipelined, lanes | 120 | 15.7 | 1.04 |
| 1080p-noise | threaded, lanes | 120 | 39.8 | 3.19 |
| 1080p | now, now | 180 | 81.6 | 1.55 |
| 1080p | pipelined, now | 180 | 86.7 | 1.66 |
| 1080p | threaded, now | 180 | 157.7 | 3.78 |
| 1080p | now, lanes | 180 | 89.1 | 1.43 |
| 1080p | pipelined, lanes | 180 | 87.7 | 1.35 |
| 1080p | threaded, lanes | 180 | 149.6 | 3.00 |
| 2160p-noise | now, now | 60 | 4.2 | 1.15 |
| 2160p-noise | pipelined, now | 60 | 4.2 | 1.17 |
| 2160p-noise | threaded, now | 60 | 10.6 | 3.52 |
| 2160p-noise | now, lanes | 60 | 4.3 | 1.08 |
| 2160p-noise | pipelined, lanes | 60 | 4.3 | 1.10 |
| 2160p-noise | threaded, lanes | 60 | 11.5 | 3.50 |
| 2160p | now, now | 120 | 19.2 | 1.50 |
| 2160p | pipelined, now | 120 | 19.6 | 1.47 |
| 2160p | threaded, now | 120 | 37.1 | 3.48 |
| 2160p | now, lanes | 120 | 20.4 | 1.33 |
| 2160p | pipelined, lanes | 120 | 21.1 | 1.42 |
| 2160p | threaded, lanes | 120 | 41.7 | 3.36 |
| 720p-noise | now, now | 180 | 38.0 | 1.15 |
| 720p-noise | pipelined, now | 180 | 37.7 | 1.23 |
| 720p-noise | threaded, now | 180 | 83.1 | 2.96 |
| 720p-noise | now, lanes | 180 | 38.8 | 1.08 |
| 720p-noise | pipelined, lanes | 180 | 39.2 | 1.08 |
| 720p-noise | threaded, lanes | 180 | 89.4 | 2.96 |

