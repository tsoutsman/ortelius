mod layout;

use bytemuck::{Pod, Zeroable};
use vello::wgpu;

use super::{SceneParams, to_buffer};
use crate::layer::Line;

pub(crate) struct LineRenderer {
    // _cull_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    group_0_layout: wgpu::BindGroupLayout,
    group_1_layout: wgpu::BindGroupLayout,
    is_miter: bool,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
struct PerLineParams {
    thickness: f32,
    colour: f32,
    _padding: [f32; 2],
}

impl LineRenderer {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let group_0_layout = SceneParams::group_layout(device, "line");
        let group_1_layout = layout::group1_layout(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render line pipeline layout"),
            bind_group_layouts: &[&group_0_layout, &group_1_layout],
            push_constant_ranges: &[],
        });
        let is_miter = false;

        LineRenderer {
            // _cull_pipeline: Line::cull_pipeline(device, &pipeline_layout),
            render_pipeline: layout::render_pipeline(device, &pipeline_layout, is_miter),
            group_0_layout,
            group_1_layout,
            is_miter,
        }
    }

    fn create_group1(&self, device: &wgpu::Device, line: Line<'_>) -> wgpu::BindGroup {
        let thickness_buffer = to_buffer(device, &[line.thickness, 0., 0., 0.]);
        let colour_buffer = to_buffer(device, &line.colour);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("line bind group 1"),
            layout: &self.group_1_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: line.data.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: thickness_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: colour_buffer.as_entire_binding(),
                },
            ],
        })
    }

    pub(crate) fn render<'a, I>(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        msaa_view: &wgpu::TextureView,
        scene_params: SceneParams,
        lines: I,
    ) where
        I: Iterator<Item = Line<'a>>,
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("line render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: msaa_view,
                resolve_target: Some(view),
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);

        let bind_group0 = scene_params.create_group(device, &self.group_0_layout, "line");
        render_pass.set_bind_group(0, &bind_group0, &[]);

        for line in lines {
            let bind_group1 = self.create_group1(device, line);
            render_pass.set_bind_group(1, &bind_group1, &[]);

            if self.is_miter {
                render_pass.draw(0..(line.data.len() * 2) as u32, 0..1);
            } else {
                render_pass.draw(0..6, 0..(line.data.len() as u32).saturating_sub(1));
            }
        }
    }
}
