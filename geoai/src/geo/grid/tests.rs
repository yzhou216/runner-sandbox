// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use super::Grid;

fn grid() -> Grid {
    Grid::new(4, 3)
        .with_origin(10.0, 50.0)
        .with_pixel_size(0.5, 0.25)
}

#[test]
fn pixel_center_round_trips() {
    let grid = grid();
    (0..grid.len()).for_each(|index| {
        let (x, y) = grid.pixel_center(index);
        assert_eq!(grid.index_of(x, y), Some(index));
    });
}

/// Each pixel center of a grid twice as coarse lies on the border of two
/// pixels, and must fall in the same one of them every time, however large
/// the coordinates are next to the pixels.
#[test]
fn index_of_breaks_ties_consistently() {
    [((10.0, 50.0), 0.1), ((500_000.0, 9_000_000.0), 0.05)]
        .into_iter()
        .for_each(|((west, north), size)| {
            let fine = Grid::new(200, 200)
                .with_origin(west, north)
                .with_pixel_size(size, size);
            let coarse = Grid::new(100, 100)
                .with_origin(west, north)
                .with_pixel_size(2.0 * size, 2.0 * size);
            let inconsistent = (0..coarse.len())
                .filter(|&index| {
                    let (x, y) = coarse.pixel_center(index);
                    let (row, column) = (index / 100, index % 100);
                    fine.index_of(x, y) != Some((2 * row + 1) * 200 + 2 * column + 1)
                })
                .count();
            assert_eq!(inconsistent, 0, "pixels inconsistent at ({west}, {north})");
        });
}

/// Pixel centers of a coarse grid whose origin lies far from them, and so
/// carry the rounding of larger coordinates, still fall on the later pixel.
#[test]
fn index_of_breaks_ties_for_distant_grids() {
    let fine = Grid::new(200, 2)
        .with_origin(0.0, 1.0)
        .with_pixel_size(0.005, 0.005);
    let coarse = Grid::new(18_100, 1)
        .with_origin(-180.0, 1.0)
        .with_pixel_size(0.01, 0.01);
    let inconsistent = (18_000..18_100)
        .filter(|&column| {
            let (x, y) = coarse.pixel_center(column);
            fine.index_of(x, y) != Some(200 + 2 * (column - 18_000) + 1)
        })
        .count();
    assert_eq!(inconsistent, 0);
}

#[test]
fn index_of_rejects_outside() {
    let grid = grid();
    assert_eq!(grid.index_of(9.99, 49.9), None);
    assert_eq!(grid.index_of(12.0, 49.9), None);
    assert_eq!(grid.index_of(10.1, 49.25), None);
    assert_eq!(grid.bounds().south, 49.25);
}
