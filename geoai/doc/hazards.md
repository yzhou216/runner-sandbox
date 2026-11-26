<!--
SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# Hazards

A hazard computes a raster from named inputs, such as earthquake hazard
classes from a raster of peak ground acceleration.  It is defined in a Scheme
file and computed with `geoai run`.  The [glossary](glossary.md) explains the
terms used here.

## Running a hazard

```sh
geoai list
geoai run earthquake earthquake.tif --input pga=PGA_250y.tif
```

`geoai list` shows the inputs of each hazard: a raster input by its name, an
input of another kind as `name:kind`, and an optional input in brackets.
Each `--input NAME=VALUE` supplies one input, either a raster file or a
finite number, according to the kind of the input.  `geoai info` prints the
grid, nodata value, and statistics of a raster file, such as one that
`geoai run` wrote.  A raster file is named by a local path, or by a URL such
as `s3://bucket/flood.tif` if it is kept in an object store, as
[Storage](storage.md) explains.

The output takes the grid of the raster given with `--like`, or else that of
the first raster input given, in the order of the signature.  Raster inputs
are sampled at the center of each output pixel and must be in the CRS of the
output, because geoai does not yet reproject rasters.  geoai knows a CRS by
its EPSG code, so a CRS defined by its parameters instead counts as unknown:
it is neither checked nor written to the output.

GeoTIFF is currently the only supported format.  An input may hold integers
of 8, 16, or 32 bits or floating-point numbers of 32 or 64 bits,
uncompressed or compressed with LZW, Deflate, PackBits, or JPEG.  Input
files are recognized by their content, whatever their names, and the output
format follows the extension of the output file, `.tif` or `.tiff` for
GeoTIFF.

## Locating definitions

geoai loads definitions in this order:

1. The built-in definitions, compiled into geoai from `hazards/` in the
   source tree.
2. Each `.scm` file in `$XDG_CONFIG_HOME/geoai/hazards`, or in
   `~/.config/geoai/hazards` if that variable is unset or not an absolute
   path.  This holds on every POSIX system, macOS included; Windows uses
   `%APPDATA%\geoai\hazards`.
3. Each path given with `--hazards`, either a file or a directory of `.scm`
   files.

A definition replaces any earlier one of the same name, so a user file can
override a built-in hazard.  Files in a directory load in name order, each in
its own Scheme environment.  Hidden files, whose names start with a dot, are
skipped.

Loading a file runs it as a Scheme program with the permissions of the user,
so load only files you trust.  A file that fails to load stops `geoai list`
and `geoai run` until it is fixed or removed.

## Defining a hazard

Hazard files are Scheme programs, made of S-expressions: atoms such as the
number `50`, the string `"earthquake_class"`, or the symbol `pga`, and lists
such as `(< pga 50)`, which calls the procedure `<` with `pga` and `50`.
Quoted, as `'(< pga 50)`, the same list is data instead of a call.

A definition takes effect in three steps:

1. When geoai loads the file, `define-hazard` records the signature and the
   declarations as data, and makes the body a procedure whose parameters are
   the inputs.
2. When `geoai run` computes the hazard, it calls that procedure once with the
   values of the inputs, and the procedure returns the raster of the hazard.
3. geoai then computes the raster at every pixel.  `pixelwise` quasiquotes its
   expression instead of evaluating it, and geoai evaluates that data at each
   pixel, so no Scheme procedure is called per pixel.

As in any quasiquote, `,` unquotes, inserting the value of a Scheme
expression, as [Using Scheme in a definition](#using-scheme-in-a-definition)
shows.

The earthquake hazard built into geoai is defined as follows:

```scheme
(define-hazard (earthquake pga (optional land))
  (type u8)
  (description "earthquake_class")
  (pixelwise (pga land)
    (cond ((missing? pga) (if (missing? land) 0 1))
          ((< pga 50) 2)
          ((< pga 150) 3)
          ((< pga 300) 4)
          (else 5))))
```

The signature names the hazard and its inputs, which become the parameters of
the body.  An input is written `name` for a raster or `(name kind)` for
another kind, where the kinds are `raster` and `number`.  An input written
`(optional name)` or `(optional name kind)` may be left unbound.

Optional declarations follow the signature, before the body, each at most
once:

- `(type u8)` or `(type f32)` sets the sample type of the output, `f32` by
  default.  A `u8` output drops the fractional part of each value, and a
  value outside the range of the type is an error.
- `(description "text")` sets the band description, the hazard name by
  default.
- `(nodata number)` sets the value written where the result is missing,
  which should be one the hazard never computes.  Without it, a missing
  result is an error.

The remaining forms are the body.  Each parameter is bound to the value of its
input: a raster, which is opaque to Scheme, a number, or `#f` for an optional
input left unbound.  The body returns the raster of the hazard, usually one
built by `pixelwise`.

## Computing pixel by pixel

`(pixelwise (name ...) expression)` returns a raster whose value at each pixel
is that of `expression` there.  Each name must be bound to a raster, a
finite number, or `#f`, and stands in the expression for the value of the
raster at that pixel, the number, or a missing value.  The expression is not
Scheme code: besides finite numbers and these names, it may contain only
these forms:

- `(+ a b ...)`, `(- a b ...)`, `(* a b ...)`, `(/ a b ...)`, `(min a b ...)`,
  and `(max a b ...)`, each with at least two operands.
- `(if condition a b)` and `(cond (condition a) ... (else b))`.

A condition is `(< a b)`, `(<= a b)`, `(> a b)`, `(>= a b)`, `(= a b)`,
`(missing? a)`, or a combination of conditions with `not`, `and`, and `or`.

A value is missing outside the grid of its raster, wherever the raster holds
nodata, and wherever it is not a finite number, such as NaN or a quotient by
zero.  A missing operand makes the result missing, so test a value with
`missing?` before using it.  `and` and `or` evaluate their operands from left
to right and stop at the first one that decides the result.

A raster of 32-bit floating-point numbers holds the nearest such number to
each value written to it, so a value written as 0.7 is slightly below 0.7,
and `(>= depth 0.7)` does not hold there.  Place thresholds between the
values that matter rather than on them.

A raster built by `pixelwise` can be an input of another `pixelwise`.

## Using Scheme in a definition

Within a `pixelwise` expression, `,expression` inserts the value of a Scheme
expression, and `,@expression` splices in the elements of a list.  Both are
evaluated once, when `pixelwise` runs.  The body may also define variables
and procedures, and a file may `require` other files relative to its own
location.

```scheme
(define-hazard (flood depth (threshold number))
  (type u8)
  (nodata 255)
  (pixelwise (depth threshold)
    (cond ((> depth ,(* 2 threshold)) 2)
          ((> depth threshold) 1)
          (else 0))))
```

Saved as `flood.scm`, the hazard runs with:

```sh
geoai run flood flood.tif --hazards flood.scm \
  --input depth=depth.tif --input threshold=0.5
```

A procedure can also build part of an expression for `,` to insert:

```scheme
(define (exceeds name threshold)
  `(> ,name ,threshold))

(define-hazard (wet depth)
  (type u8)
  (nodata 255)
  (pixelwise (depth) (if ,(exceeds 'depth 0.5) 1 0)))
```
