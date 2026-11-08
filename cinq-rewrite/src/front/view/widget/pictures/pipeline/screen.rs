// SPDX-FileCopyrightText: 2026 Yiyu Zhou <yiyu@yiyuzhou.io>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Texture of the pictures of one cue, which each new picture fills again

use iced::{Size, advanced::image, wgpu};

use super::{PLACEMENT, Pipeline};

pub(super) struct Screen {
    /// Picture that the texture holds, once it holds one
    picture: Option<image::Id>,
    pub(super) texture: wgpu::Texture,
    placement: wgpu::Buffer,
    pub(super) bindings: wgpu::BindGroup,
}

impl Screen {
    pub(super) fn new(pipeline: &Pipeline, device: &wgpu::Device, extent: wgpu::Extent3d) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cinq picture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: pipeline.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let placement = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cinq picture placement"),
            size: PLACEMENT,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bindings = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cinq picture"),
            layout: &pipeline.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: placement.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&pipeline.sampler),
                },
            ],
        });
        Self {
            picture: None,
            texture,
            placement,
            bindings,
        }
    }

    pub(super) fn filled(
        self,
        queue: &wgpu::Queue,
        picture: &image::Handle,
        pixels: &[u8],
    ) -> Self {
        if self.picture == Some(picture.id()) {
            return self;
        }
        let extent = self.texture.size();
        queue.write_texture(
            self.texture.as_image_copy(),
            pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * extent.width),
                rows_per_image: Some(extent.height),
            },
            extent,
        );
        Self {
            picture: Some(picture.id()),
            ..self
        }
    }

    /// Places the picture in the middle of its region, spanning the share
    /// `span` of its width and height
    pub(super) fn place(&self, queue: &wgpu::Queue, span: Size, opacity: f32) {
        let placement: Vec<u8> = [span.width, span.height, opacity, 0.0]
            .into_iter()
            .flat_map(f32::to_ne_bytes)
            .collect();
        queue.write_buffer(&self.placement, 0, &placement);
    }
}
