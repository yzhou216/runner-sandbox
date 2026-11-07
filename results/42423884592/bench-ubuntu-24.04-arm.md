# ubuntu-24.04-arm

CPU(s):                                  4
Model name:                              Neoverse-N2

## Kernels of the conversion, 4 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 0.53 | 0.17 | 0.62 |
| 640x360 | pairs | yes | 0.27 | 0.10 | 0.37 |
| 640x360 | lanes | yes | 0.24 | 0.09 | 0.34 |
| 1280x720 | now | yes | 2.14 | 0.62 | 2.29 |
| 1280x720 | pairs | yes | 1.14 | 0.36 | 1.27 |
| 1280x720 | lanes | yes | 1.03 | 0.33 | 1.12 |
| 1920x1080 | now | yes | 4.80 | 1.35 | 4.93 |
| 1920x1080 | pairs | yes | 2.55 | 0.80 | 2.72 |
| 1920x1080 | lanes | yes | 2.29 | 0.72 | 2.42 |
| 3840x2160 | now | yes | 19.32 | 5.33 | 19.44 |
| 3840x2160 | pairs | yes | 10.22 | 3.06 | 10.44 |
| 3840x2160 | lanes | yes | 9.17 | 2.76 | 9.31 |

## Passes through whole clips, 4 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p-noise | now, now | 120 | 15.6 | 1.05 |
| 1080p-noise | pipelined, now | 120 | 16.1 | 1.08 |
| 1080p-noise | threaded, now | 120 | 47.4 | 3.22 |
| 1080p-noise | now, lanes | 120 | 15.8 | 1.03 |
| 1080p-noise | pipelined, lanes | 120 | 16.1 | 1.04 |
| 1080p-noise | threaded, lanes | 120 | 48.7 | 3.19 |
| 1080p | now, now | 180 | 79.2 | 1.25 |
| 1080p | pipelined, now | 180 | 96.2 | 1.49 |
| 1080p | threaded, now | 180 | 200.5 | 3.37 |
| 1080p | now, lanes | 180 | 84.3 | 1.13 |
| 1080p | pipelined, lanes | 180 | 93.5 | 1.24 |
| 1080p | threaded, lanes | 180 | 221.8 | 3.17 |
| 2160p-noise | now, now | 60 | 4.4 | 1.06 |
| 2160p-noise | pipelined, now | 60 | 4.5 | 1.09 |
| 2160p-noise | threaded, now | 60 | 14.0 | 3.44 |
| 2160p-noise | now, lanes | 60 | 4.4 | 1.03 |
| 2160p-noise | pipelined, lanes | 60 | 4.5 | 1.04 |
| 2160p-noise | threaded, lanes | 60 | 14.6 | 3.42 |
| 2160p | now, now | 120 | 20.4 | 1.28 |
| 2160p | pipelined, now | 120 | 23.3 | 1.46 |
| 2160p | threaded, now | 120 | 51.8 | 3.39 |
| 2160p | now, lanes | 120 | 21.6 | 1.14 |
| 2160p | pipelined, lanes | 120 | 23.2 | 1.22 |
| 2160p | threaded, lanes | 120 | 56.2 | 3.12 |
| 720p-noise | now, now | 180 | 36.8 | 1.05 |
| 720p-noise | pipelined, now | 180 | 38.6 | 1.09 |
| 720p-noise | threaded, now | 180 | 98.5 | 2.85 |
| 720p-noise | now, lanes | 180 | 37.5 | 1.03 |
| 720p-noise | pipelined, lanes | 180 | 38.6 | 1.05 |
| 720p-noise | threaded, lanes | 180 | 100.8 | 2.80 |

