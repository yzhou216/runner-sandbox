# ubuntu-latest

CPU(s):                                  4
Model name:                              AMD EPYC 7763 64-Core Processor

## Kernels of the conversion, 4 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 2.05 | 0.69 | 2.75 |
| 640x360 | pairs | yes | 0.52 | 0.27 | 1.09 |
| 640x360 | lanes | yes | 0.49 | 0.23 | 0.94 |
| 1280x720 | now | yes | 8.25 | 2.75 | 10.78 |
| 1280x720 | pairs | yes | 2.06 | 1.08 | 4.06 |
| 1280x720 | lanes | yes | 2.00 | 0.92 | 3.44 |
| 1920x1080 | now | yes | 18.60 | 6.16 | 24.02 |
| 1920x1080 | pairs | yes | 4.64 | 2.38 | 8.85 |
| 1920x1080 | lanes | yes | 4.48 | 2.03 | 7.60 |
| 3840x2160 | now | yes | 75.43 | 25.43 | 96.27 |
| 3840x2160 | pairs | yes | 19.15 | 10.01 | 36.00 |
| 3840x2160 | lanes | yes | 18.66 | 11.10 | 28.35 |

## Passes through whole clips, 4 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p-noise | now, now | 120 | 10.8 | 1.14 |
| 1080p-noise | pipelined, now | 120 | 11.2 | 1.20 |
| 1080p-noise | threaded, now | 120 | 25.0 | 3.34 |
| 1080p-noise | now, lanes | 120 | 11.1 | 1.06 |
| 1080p-noise | pipelined, lanes | 120 | 11.4 | 1.09 |
| 1080p-noise | threaded, lanes | 120 | 27.1 | 3.32 |
| 1080p | now, now | 180 | 57.4 | 1.58 |
| 1080p | pipelined, now | 180 | 66.1 | 1.86 |
| 1080p | threaded, now | 180 | 100.9 | 3.52 |
| 1080p | now, lanes | 180 | 63.0 | 1.34 |
| 1080p | pipelined, lanes | 180 | 68.8 | 1.53 |
| 1080p | threaded, lanes | 180 | 115.5 | 3.30 |
| 2160p-noise | now, now | 60 | 2.9 | 1.15 |
| 2160p-noise | pipelined, now | 60 | 3.0 | 1.20 |
| 2160p-noise | threaded, now | 60 | 6.9 | 3.52 |
| 2160p-noise | now, lanes | 60 | 3.0 | 1.07 |
| 2160p-noise | pipelined, lanes | 60 | 3.0 | 1.09 |
| 2160p-noise | threaded, lanes | 60 | 7.4 | 3.47 |
| 2160p | now, now | 120 | 14.1 | 1.56 |
| 2160p | pipelined, now | 120 | 16.5 | 1.84 |
| 2160p | threaded, now | 120 | 25.7 | 3.59 |
| 2160p | now, lanes | 120 | 14.9 | 1.33 |
| 2160p | pipelined, lanes | 120 | 17.0 | 1.49 |
| 2160p | threaded, lanes | 120 | 29.6 | 3.37 |
| 720p-noise | now, now | 180 | 25.3 | 1.14 |
| 720p-noise | pipelined, now | 180 | 26.3 | 1.22 |
| 720p-noise | threaded, now | 180 | 54.5 | 3.01 |
| 720p-noise | now, lanes | 180 | 26.1 | 1.06 |
| 720p-noise | pipelined, lanes | 180 | 26.8 | 1.10 |
| 720p-noise | threaded, lanes | 180 | 56.9 | 2.86 |

