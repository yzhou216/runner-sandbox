<!--
SPDX-FileCopyrightText: 2026 ParrotGaming <31054631+ParrotGaming@users.noreply.github.com>

SPDX-License-Identifier: 0BSD OR CC0-1.0
-->

# Python bindings plan

This plan adds PyO3 bindings that expose the raw transformation operations
of geoai, so that pipelines can be built in Python scripts.

The core already has a clean seam for this.  `geo::Layer` is `Send + Sync`
and moves around as `Arc<dyn Layer>`.  `pixelwise::Pixelwise::new` takes a
language-neutral `Sexp`.  `hazard::Body` is documented as the place where a
new language plugs in.  So the bindings can be a thin layer over existing
code, with only small changes to the core.

## 1. Crate layout

Put the bindings in a separate crate in a Cargo workspace.  Don't add a
`python` feature to the main crate.

```text
Cargo.toml              # [workspace] members = [".", "python"]
python/
  Cargo.toml            # geoai-py: cdylib; geoai (path = ".."), pyo3, numpy
  pyproject.toml        # maturin build backend, module-name = "geoai._geoai"
  src/lib.rs            # #[pymodule] _geoai
  src/{grid,layer,raster,expr,io,hazard,error}.rs
  geoai/__init__.py     # re-exports from _geoai, plus pure-Python helpers
  geoai/__init__.pyi    # type stubs
  tests/test_*.py       # pytest, reusing ../tests/data
```

Why a separate crate:

- The main crate stays a plain `rlib` and binary, with no PyO3 in its build.
- The rule in `doc/architecture.md` ("another language could define hazards
  by implementing `hazard::Body`") then applies literally: Python is just
  another front end next to `script`.
- Clippy and tests for each crate stay independent.

Use `abi3` (for example `abi3-py310`) so one wheel works across Python
versions, and turn on PyO3's `anyhow` feature.

## 2. The Python objects

- `Crs` wraps `geo::Crs`: `Crs.epsg(4326)`, `.code`, `__eq__`, `__repr__`.
- `Grid` wraps `geo::Grid`.  It is immutable: `with_origin`,
  `with_pixel_size`, and `with_crs` return new grids.  It has `.bounds`.
- `Layer` wraps `Arc<dyn Layer>` as a `#[pyclass(frozen)]`.  It is lazy and
  is the main value that pipelines pass around.
- `Expr` and `Cond` wrap a small Rust tree whose leaves are
  `Arc<dyn Layer>`.  Python operators build them.
- `Raster` wraps `enum { U8(Raster<u8>), F32(Raster<f32>) }`, the
  materialized result, with `.grid`, `.nodata`, `.stats()`, and
  `.to_numpy()`.
- `Hazard` and `Catalog` wrap `hazard::Hazard` and `catalog::Catalog`.  They
  must be `#[pyclass(unsendable)]`, because a Steel `Procedure` holds an
  `Rc<Environment>`.

A `Raster` should also be usable as a `Layer`, since `Raster<T>` implements
`Layer`.  Either give it an `.as_layer()` method or accept it wherever a
`Layer` is expected.

## 3. Operations to expose

```python
geoai.read(path) -> Layer        # format::read_layer, plus its grid
geoai.read_grid(path) -> Grid    # format::read_grid
geoai.constant(x) -> Layer       # geo::Constant
geoai.pixelwise(expr) -> Layer   # pixelwise::Pixelwise
layer.compute(grid, dtype="f32", nodata=None) -> Raster
                                 # Raster::from_layer
geoai.write(raster, path, description=...)
                                 # format::write
geoai.Raster.from_numpy(array, grid, nodata=None)
                                 # Raster::new
```

`compute` runs `Raster::from_layer`, which samples every pixel in parallel
with rayon.  Wrap that call in `py.detach(...)` (it was called
`allow_threads` before PyO3 0.26) so other Python threads can run during the
computation.  That is safe because every `Layer` is `Send + Sync`.

## 4. Building pixelwise expressions from Python

This is the main design decision.  Python operators build a deferred
expression tree.  `pixelwise()` lowers that tree to a `Sexp` and calls the
existing `Pixelwise::new`.

```python
from geoai import read, pixelwise, missing, cond, if_

pga = read("tests/data/PGA_250y.tif")
land = None  # an optional input, missing everywhere

hazard = pixelwise(cond(
    (missing(pga), if_(missing(land), 0, 1)),
    (pga < 50, 2),
    (pga < 150, 3),
    (pga < 300, 4),
    else_=5,
))
geoai.write(hazard.compute(pga.grid, dtype="u8"), "earthquake.tif",
            description="earthquake_class")
```

How it works:

- `Layer` and `Expr` both implement `__add__`, `__sub__`, `__mul__`,
  `__truediv__`, and their reflected versions, which return an `Expr`.
- `__lt__`, `__le__`, `__gt__`, `__ge__`, and `__eq__` return a `Cond`.
- `&`, `|`, and `~` on a `Cond` build `and`, `or`, and `not`.
- `minimum`, `maximum`, `missing`, `if_`, and `cond` are functions.
- When it lowers the tree, `pixelwise()` gives each distinct leaf layer a
  generated name (`_0`, `_1`, and so on), deduplicated with `Arc::ptr_eq`.
  It then emits the same `Sexp` that the Scheme `pixelwise` macro emits.
  `None` becomes a binding of `None`, which is missing everywhere.
- `Cond.__bool__` raises an error.  That catches `pga < 50 and x` and
  chained comparisons, which Python can't overload.  numpy does the same.

Why not have each operator return a new `Pixelwise` layer right away?  Two
reasons.  First, every nested layer would sample its own inputs again, so a
value like `pga` would be sampled many times per pixel.  Second, `cond`,
`and`, and `or` short-circuit only inside a single `Expression`.  A deferred
tree with one `Pixelwise` at the end keeps the current guarantee that each
input is sampled once per pixel.

Why lower to `Sexp` rather than make `pixelwise::expression` public?  `Sexp`
is already the documented boundary.  Python and Scheme then share the same
parser checks and the same error messages, and the expression enums stay
private.

Users can still compose these.  `pixelwise(...)` returns a `Layer`, which
can be a leaf in another expression, so a whole pipeline is a lazy graph of
layers until `compute()` runs.

## 5. Rules that must hold

- No Python code runs per pixel.  Don't offer a `Layer` backed by a Python
  callback.  Inside the parallel loop of `from_layer`, each sample would need
  the GIL, so the work would run one pixel at a time or deadlock.  Custom
  per-pixel logic either goes through `Expr` or is materialized with numpy
  and passed back through `Raster.from_numpy`.
- Convert every `anyhow::Error` into a `geoai.GeoaiError` exception (a
  subclass of `RuntimeError`) formatted with `{:#}`.  That keeps the context
  chains the core builds, such as "cannot read X: ...".
- A `Hazard` or `Catalog` raises an error if it is used from another Python
  thread.  `Layer`, `Grid`, and `Raster` have no such limit.

## 6. Changes needed in the core crate

These are small:

1. Read the grid and the layer together.  `format::read_layer` drops the
   grid, and calling `read_grid` afterwards opens the file twice.
   `geotiff::decode` already has both, so add a function that returns
   `(Grid, Arc<dyn Layer>)`.
2. Add a public constructor for `Hazard`.  It is needed only in phase 5, for
   bodies not written in Scheme.  Today the only way to build one is
   `Hazard::parse` from a `Sexp`.
3. Optionally, add a `Sexp` reader (`impl FromStr for Sexp`).  It would let
   `pixelwise({"pga": pga}, "(cond ...)")` take the same text as Scheme
   files, and could help the CLI later.
4. Optionally, put `cli` and `clap` behind a default feature so the Python
   extension doesn't link them.

## 7. Hazards from Python

This comes in a later phase.

- Run Scheme hazards from Python: `Catalog.load(paths=[...])` and
  `catalog["earthquake"].evaluate(grid, pga=layer)`.  These wrap
  `Hazard::bind` and `Hazard::evaluate`, choosing `u8` or `f32` from
  `sample_type()` the way `src/cli/run.rs` does.
- Define hazards in Python: add a `PyBody` that implements `hazard::Body` by
  calling a Python function.  That function runs once per evaluation and
  returns a `Layer`, exactly like a Scheme body, so it doesn't break the "no
  Python per pixel" rule.  A decorator such as
  `@geoai.hazard(type="u8", inputs=["pga", optional("land")])` could then
  register Python hazards next to Scheme ones.

## 8. Build, CI, and repository rules

- In `flake.nix`, add `packages.python`, built with
  `python3Packages.buildPythonPackage` and `maturinBuildHook` using the
  shared `Cargo.lock`.  Add a `checks.python-tests` that runs pytest against
  `tests/data`, and make sure `cargo-clippy` covers the workspace.  Add
  `maturin` and `python3` with `pytest` and `numpy` to the dev shell.
- Every new file needs SPDX headers so `reuse lint` passes.  The Python
  package is AGPL-3.0-or-later, like the crate.
- Update `doc/architecture.md`: add a `python` front end to the module list,
  and add "a Python operation" to "Where changes belong".
- Keep the PyO3 and `numpy` crate versions pinned to a pair that work
  together, because each `numpy` release targets one PyO3 minor version.

## 9. Order of work

1. Scaffolding: the workspace, the crate, maturin, the Nix check, and a smoke
   test that imports the module.
2. Basic types and I/O: `Crs`, `Grid`, `Layer`, `Raster`, `read`, `write`,
   `constant`, and `compute` with the GIL released.
3. Expressions: `Expr`, `Cond`, and `pixelwise`.  The test is to reproduce
   `hazards/earthquake.scm` in Python and compare the output with
   `tests/data/hazard_earthquake.tif`.
4. numpy: `Raster.to_numpy()` and `Raster.from_numpy()`.
5. Hazards: `Catalog` and `Hazard` from Python, then `PyBody`.
6. Polish: type stubs, docs, and tests for error cases.

Steps 1 to 3 already give complete pipelines in Python scripts.
