# windows-latest

AMD EPYC 9V45 96-Core Processor                
4

## Kernels of the conversion, 4 strips

- now at 641x361: same bytes
- pairs at 641x361: same bytes
- lanes at 641x361: same bytes

| size | kernel | same bytes | serial ms | rayon-strips ms | rayon-strips CPU ms |
| --- | --- | --- | --- | --- | --- |
| 640x360 | now | yes | 1.84 | 0.73 | 1.99 |
| 640x360 | pairs | yes | 0.41 | 0.38 | 0.82 |
| 640x360 | lanes | yes | 0.43 | 0.27 | 0.94 |
| 1280x720 | now | yes | 7.01 | 2.28 | 8.57 |
| 1280x720 | pairs | yes | 1.90 | 0.95 | 3.05 |
| 1280x720 | lanes | yes | 1.83 | 0.85 | 2.85 |
| 1920x1080 | now | yes | 15.90 | 5.12 | 18.94 |
| 1920x1080 | pairs | yes | 4.30 | 2.06 | 7.59 |
| 1920x1080 | lanes | yes | 4.17 | 1.85 | 6.37 |
| 3840x2160 | now | yes | 64.70 | 20.99 | 83.93 |
| 3840x2160 | pairs | yes | 17.39 | 8.05 | 29.32 |
| 3840x2160 | lanes | yes | 16.75 | 7.05 | 24.63 |

## cinq's conversion of high definition, at ec61ead and in the snapshot

| size | ec61ead ms | ec61ead CPU ms | snapshot ms | snapshot CPU ms | same bytes |
| --- | --- | --- | --- | --- | --- |
| 1280x720 | 2.45 | 8.56 | 1.20 | 4.41 | yes |
| 1920x1080 | 5.21 | 15.31 | 2.48 | 8.91 | yes |
| 3840x2160 | 20.42 | 76.92 | 9.44 | 34.49 | yes |

## Passes through whole clips, 4 cores

| clip | variant | pictures | pictures a second | cores busy |
| --- | --- | --- | --- | --- |
| 1080p | now, now | 180 | 69.8 | 1.36 |
| 1080p | now, pairs | 180 | 78.1 | 1.40 |
| 1080p | pipelined, pairs | 180 | 82.2 | 1.53 |
| 1080p | threaded, now | 180 | 144.7 | 3.72 |
| 1080p | threaded, pairs | 180 | 156.6 | 3.52 |
| 1080p30-grain | now, now | 180 | 35.8 | 1.30 |
| 1080p30-grain | now, pairs | 180 | 36.6 | 1.20 |
| 1080p30-grain | pipelined, pairs | 180 | 36.1 | 1.22 |
| 1080p30-grain | threaded, now | 180 | 68.9 | 3.02 |
| 1080p30-grain | threaded, pairs | 180 | 72.3 | 2.79 |
| 1080p60-grain | now, now | 240 | 48.0 | 1.39 |
| 1080p60-grain | now, pairs | 240 | 49.2 | 1.33 |
| 1080p60-grain | pipelined, pairs | 240 | 51.8 | 1.37 |
| 1080p60-grain | threaded, now | 240 | 95.3 | 3.23 |
| 1080p60-grain | threaded, pairs | 240 | 100.1 | 3.11 |
| 2160p | now, now | 120 | 17.0 | 1.48 |
| 2160p | now, pairs | 120 | 17.7 | 1.39 |
| 2160p | pipelined, pairs | 120 | 18.5 | 1.44 |
| 2160p | threaded, now | 120 | 33.7 | 3.48 |
| 2160p | threaded, pairs | 120 | 36.9 | 3.27 |
| 2160p30-grain | now, now | 90 | 11.6 | 1.35 |
| 2160p30-grain | now, pairs | 90 | 12.0 | 1.23 |
| 2160p30-grain | pipelined, pairs | 90 | 12.1 | 1.28 |
| 2160p30-grain | threaded, now | 90 | 21.7 | 2.98 |
| 2160p30-grain | threaded, pairs | 90 | 23.5 | 2.87 |

