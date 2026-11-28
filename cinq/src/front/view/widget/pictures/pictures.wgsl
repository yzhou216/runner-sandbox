// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

// Picture of a cue in the middle of its region, at its opacity

struct Placement {
    // Share of the width and of the height of the region that it spans
    span: vec2<f32>,
    opacity: f32,
}

@group(0) @binding(0) var<uniform> placement: Placement;
@group(0) @binding(1) var picture: texture_2d<f32>;
@group(0) @binding(2) var picture_sampler: sampler;

struct Corner {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

// Corners from the top left to the bottom right, as a strip of two
// triangles
@vertex
fn vs_main(@builtin(vertex_index) index: u32) -> Corner {
    let uv = vec2<f32>(f32(index & 1u), f32(index >> 1u));
    let position = (uv * 2.0 - 1.0) * vec2<f32>(placement.span.x, -placement.span.y);
    return Corner(vec4<f32>(position, 0.0, 1.0), uv);
}

@fragment
fn fs_main(corner: Corner) -> @location(0) vec4<f32> {
    return textureSample(picture, picture_sampler, corner.uv) * placement.opacity;
}
