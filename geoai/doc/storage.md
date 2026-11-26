<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# Storage

geoai reads and writes rasters as files, either in the local filesystem or
in an object store such as S3.  This document explains how to name such
files, and how the geoai service is to keep rasters and their metadata,
whether it is hosted for its users or hosted by them.

## Paths and URLs

Wherever geoai takes a raster file, it takes a local path or a URL:

```sh
geoai run earthquake s3://hazards/earthquake.tif \
  --input pga=https://example.org/PGA_250y.tif
geoai info s3://hazards/earthquake.tif
```

| URL                             | Store                               |
| ------------------------------- | ----------------------------------- |
| `file:///data/flood.tif`        | Local filesystem, as a plain path   |
| `s3://bucket/flood.tif`         | S3, or any store compatible with it |
| `gs://bucket/flood.tif`         | Google Cloud Storage                |
| `az://container/flood.tif`      | Azure Blob Storage                  |
| `https://example.org/flood.tif` | Web server                          |

Text containing `://` is a URL, and any other text is a path.  Each store
reads its settings and credentials from the environment variables that
start with `AWS_`, `GOOGLE_`, or `AZURE_`, as the builders of the
`object_store` crate document.  A store compatible with S3, such as Garage
or Ceph, typically needs these:

```sh
export AWS_ENDPOINT=https://storage.example.org
export AWS_REGION=garage
export AWS_ACCESS_KEY_ID=...
export AWS_SECRET_ACCESS_KEY=...
```

`AWS_ALLOW_HTTP=true` permits an endpoint without TLS, such as a store on
the same machine.  A store reached over TLS requires the CA certificates of
the system, which a minimal container image may lack; `SSL_CERT_FILE` names
a file of them elsewhere.

geoai downloads an object whole before decoding it, even when it needs only
the grid, as from the raster given with `--like`.  The first raster input,
when it gives the output its grid, is therefore downloaded once for the grid
and again for its pixels.  geoai uploads an output once it is computed, in
parts if it is large.

## The service

geoai is to be offered as a hosted service and to be self-hostable, from
the same code.  The service is not written yet; this section records its
design for storage.

Raster data lives in an object store or on a local disk, never in a
database.  A single band of 32-bit numbers on the grid of the earthquake
test data takes 314 MB.  In a database, such files would swell its backups
and replicas, and every read would pass through the database server.  An
object store serves them directly, and GDAL and QGIS read parts of a file
from it with HTTP range requests.

Metadata lives in PostgreSQL with PostGIS:

- Tenants, their users, and their quotas.
- Datasets: the URL, checksum, size, grid, and CRS of each file, and the
  area it covers.
- Runs: the hazard, the checksum of its Scheme source, the geoai version,
  the input datasets and numbers, the output dataset, the state of the run,
  and any error.

PostgreSQL makes the state of a run transactional, and serves as the queue
of runs through `SELECT ... FOR UPDATE SKIP LOCKED`, so the service needs no
separate message broker.  PostGIS answers spatial questions, such as which
outputs cover an area.  A self-hosted instance therefore needs only
PostgreSQL and a place for files.  Supporting a single database keeps every
query and test single too; a lone user needs no database, since the
command line works on files directly.

The service follows these rules:

1. Objects are written once and never replaced.  A new run writes new
   objects, so no cache or row goes stale, and a run can always be repeated
   from the inputs it names.
2. An object is written before the row that names it, and a row is deleted
   before its object.  A failure in between leaves an orphaned object, which
   a periodic sweep removes, and never a row that names nothing.
3. Each tenant has a prefix of its own, within which sources are named by
   their content:

   ```text
   tenants/<tenant>/sources/<sha256>.tif
   tenants/<tenant>/runs/<run>/<output>.tif
   ```

   Naming by content stores a source that is uploaded twice only once.  The
   prefix keeps tenants from probing for the files of others by checksum, and
   makes deleting a tenant or measuring its usage a matter of one prefix.
4. Clients never hold credentials for the store.  The service hands out
   presigned URLs for uploads and downloads, which S3, Google Cloud Storage,
   and Azure support, and streams the files itself from a local disk.
5. A worker takes a run from the queue, runs geoai with the URLs of the
   inputs and the output, and records the result.  geoai itself never
   connects to the database, so it works the same with or without the
   service.

| Deployment         | Files                                | Metadata           |
| ------------------ | ------------------------------------ | ------------------ |
| A single user      | Local disk or any object store       | None               |
| Self-hosted server | Local disk or a store such as Garage | PostgreSQL         |
| Hosted service     | S3, Google Cloud Storage, or Azure   | Managed PostgreSQL |

Two later steps build on this design.  Writing outputs as Cloud Optimized
GeoTIFFs lets web maps, and geoai itself, read only the tiles they need.
Recording datasets with the fields of STAC items lets the service publish a
STAC API, which GIS tools can search.
