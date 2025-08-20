mod layout;

use bytemuck::{Pod, Zeroable};
use vello::wgpu::{self, util::DeviceExt};

use crate::{layer::Line, render::SceneParams};

pub(crate) struct LineRenderer {
    // _cull_pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    group0_layout: wgpu::BindGroupLayout,
    group1_layout: wgpu::BindGroupLayout,
}

impl LineRenderer {
    pub(crate) fn new(device: &wgpu::Device) -> Self {
        let group0_layout = layout::group0_layout(device);
        let group1_layout = layout::group1_layout(device);

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Line Pipeline Layout"),
            bind_group_layouts: &[&group0_layout, &group1_layout],
            push_constant_ranges: &[],
        });

        LineRenderer {
            // _cull_pipeline: Line::cull_pipeline(device, &pipeline_layout),
            render_pipeline: layout::render_pipeline(device, &pipeline_layout, false),
            group0_layout,
            group1_layout,
        }
    }

    fn create_group0(
        &self,
        device: &wgpu::Device,
        scene_params: SceneParams,
    ) -> wgpu::BindGroup {
        let scene_buffer = to_buffer(device, &scene_params);
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Line Bind Group 0"),
            layout: &self.group0_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_buffer.as_entire_binding(),
            }],
        })
    }

    fn create_group1(&self, device: &wgpu::Device, line: &Line) -> wgpu::BindGroup {
        let thickness_buffer = to_buffer(device, &line.thickness);

        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Line Bind Group 1"),
            layout: &self.group1_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: line.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: thickness_buffer.as_entire_binding(),
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
        I: Iterator<Item = &'a Line>,
    {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Line Render Pass"),
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

        let bind_group0 = self.create_group0(device, scene_params);
        render_pass.set_bind_group(0, &bind_group0, &[]);

        for line in lines {
            let bind_group1 = self.create_group1(device, line);
            render_pass.set_bind_group(1, &bind_group1, &[]);
            render_pass.draw(0..(line.buffer.len() * 2) as u32, 0..1);
        }
    }
}

fn to_buffer<T>(device: &wgpu::Device, value: &T) -> wgpu::Buffer
where
    T: Pod + Zeroable,
{
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Line Thickness Buffer"),
        contents: bytemuck::bytes_of(value),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}
