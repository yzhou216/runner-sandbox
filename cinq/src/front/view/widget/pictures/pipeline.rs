// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Pictures as the GPU draws them, each cue from a texture of its own that
//! every new picture of the cue fills again
//!
//! The textures follow the pictures of the latest frame: a cue that no
//! longer shows lets go of its texture, and a cue whose pictures change
//! size gets a new one, which [`screen`] keeps.  A picture wider or taller
//! than the largest texture of the GPU does not show.

mod screen;

use std::collections::HashMap;

use iced::{
    Rectangle, Size, wgpu,
    widget::shader::{self, Viewport},
};

use super::{Layer, fitted, rgba};
use crate::back::CueId;
use screen::Screen;

/// Bytes of the placement of a picture: the share of the width and of the
/// height of its region that it spans, its opacity, and padding
const PLACEMENT: u64 = 16;

/// Pictures of a region, from the bottom up, as the GPU draws them
#[derive(Debug)]
pub struct Pictures(pub Vec<Layer>);

impl shader::Primitive for Pictures {
    type Pipeline = Pipeline;

    fn prepare(
        &self,
        pipeline: &mut Pipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        _viewport: &Viewport,
    ) {
        pipeline.prepare(device, queue, *bounds, &self.0);
    }

    fn draw(&self, pipeline: &Pipeline, pass: &mut wgpu::RenderPass<'_>) -> bool {
        pass.set_pipeline(&pipeline.render);
        self.0
            .iter()
            .filter_map(|layer| pipeline.screens.get(&layer.cue))
            .for_each(|screen| {
                pass.set_bind_group(0, &screen.bindings, &[]);
                pass.draw(0..4, 0..1);
            });
        true
    }
}

/// Shader of the pictures, along with the texture of each cue that shows
pub struct Pipeline {
    render: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    /// Format of the textures, which decodes the colors of pictures as the
    /// target encodes them
    format: wgpu::TextureFormat,
    /// Largest width and height of a texture
    largest: u32,
    screens: HashMap<CueId, Screen>,
}

impl shader::Pipeline for Pipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, target: wgpu::TextureFormat) -> Self {
        let module = device.create_shader_module(wgpu::include_wgsl!("pictures.wgsl"));
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cinq pictures"),
            entries: &[
                binding(
                    0,
                    wgpu::ShaderStages::VERTEX_FRAGMENT,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(PLACEMENT),
                    },
                ),
                binding(
                    1,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
                binding(
                    2,
                    wgpu::ShaderStages::FRAGMENT,
                    wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                ),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("cinq pictures"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let render = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cinq pictures"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target,
                    // Shader multiplies the colors by the opacity
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..wgpu::PrimitiveState::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("cinq pictures"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..wgpu::SamplerDescriptor::default()
        });
        let format = if target.is_srgb() {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };
        Self {
            render,
            layout,
            sampler,
            format,
            largest: device.limits().max_texture_dimension_2d,
            screens: HashMap::new(),
        }
    }
}

impl Pipeline {
    /// Fills the texture of each cue of `layers` with its picture, places
    /// the picture within `bounds`, and lets go of the textures of the cues
    /// that no longer show
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: Rectangle,
        layers: &[Layer],
    ) {
        let mut screens = std::mem::take(&mut self.screens);
        self.screens = layers
            .iter()
            .filter_map(|layer| {
                let (size, pixels) = rgba(&layer.picture)
                    .filter(|(size, _)| size.width.max(size.height) <= self.largest)?;
                let extent = wgpu::Extent3d {
                    width: size.width,
                    height: size.height,
                    depth_or_array_layers: 1,
                };
                let screen = screens
                    .remove(&layer.cue)
                    .filter(|screen| screen.texture.size() == extent)
                    .unwrap_or_else(|| Screen::new(self, device, extent))
                    .filled(queue, &layer.picture, pixels);
                let fitted = fitted(bounds, size);
                let span = Size::new(fitted.width / bounds.width, fitted.height / bounds.height);
                screen.place(queue, span, layer.opacity);
                Some((layer.cue, screen))
            })
            .collect();
    }
}

/// Returns the binding `index` of the shader, which `stages` see
fn binding(
    index: u32,
    stages: wgpu::ShaderStages,
    ty: wgpu::BindingType,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: index,
        visibility: stages,
        ty,
        count: None,
    }
}
