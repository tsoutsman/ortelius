use bytemuck::Pod;
use vello::wgpu::{
    self, BindingResource, BufferUsages, COPY_BUFFER_ALIGNMENT, CommandBuffer,
    CommandEncoder,
};

pub fn pad_size(size: u64) -> u64 {
    let align_mask = COPY_BUFFER_ALIGNMENT - 1;
    ((size + align_mask) & !align_mask).max(COPY_BUFFER_ALIGNMENT)
}

pub struct GpuBuffer<T> {
    length: usize,
    capacity: u64,
    inner: wgpu::Buffer,
    usage: BufferUsages,
    growth: fn(u64, u64, bool) -> u64,
    _marker: std::marker::PhantomData<T>,
}

impl<T> GpuBuffer<T>
where
    T: Pod,
{
    pub fn len(&self) -> usize {
        self.length
    }

    #[inline]
    pub fn new<F>(device: &wgpu::Device, usage: BufferUsages, length: usize, fill: F) -> Self
    where
        F: FnOnce(&mut [T]),
    {
        assert!(
            usage.contains(BufferUsages::COPY_SRC),
            "Buffer must have COPY_SRC usage"
        );
        assert!(
            usage.contains(BufferUsages::COPY_DST),
            "Buffer must have COPY_DST usage"
        );

        let growth = default_growth;
        let capacity = pad_size(growth(
            0,
            length as u64 * std::mem::size_of::<T>() as u64,
            true,
        ));

        let inner = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: capacity,
            usage,
            mapped_at_creation: true,
        });

        {
            let mut buffer_view = inner.slice(..).get_mapped_range_mut();
            let buffer_view_2 = &mut bytemuck::cast_slice_mut(buffer_view.as_mut())[..length];
            fill(buffer_view_2);
        }

        inner.unmap();
        Self {
            inner,
            length,
            capacity,
            usage,
            growth,
            _marker: std::marker::PhantomData,
        }
    }

    fn size(&self) -> u64 {
        (self.length * std::mem::size_of::<T>()) as u64
    }

    #[inline]
    fn grow(&mut self, device: &wgpu::Device, required_size: u64) -> CommandEncoder {
        let new_capacity = pad_size((self.growth)(self.capacity, required_size, false));
        assert!(
            new_capacity >= required_size,
            "New capacity must be at least as large as the required size"
        );
        assert!(
            new_capacity > self.capacity,
            "New capacity must be greater than the current capacity"
        );

        let new_inner = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: new_capacity,
            usage: self.usage,
            mapped_at_creation: false,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Grow Command Encoder"),
        });

        encoder.copy_buffer_to_buffer(&self.inner, 0, &new_inner, 0, self.size());

        self.inner = new_inner;
        self.capacity = new_capacity;

        encoder
    }

    #[inline]
    pub fn extend<F>(&mut self, device: &wgpu::Device, length: usize, fill: F) -> CommandBuffer
    where
        F: FnOnce(&mut [T]),
    {
        let extra_size = (length * std::mem::size_of::<T>()) as u64;
        let required_size = self.size() + extra_size;
        let mut encoder = if required_size > self.capacity {
            self.grow(device, required_size)
        } else {
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Append Command Encoder"),
            })
        };

        let temp = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: pad_size(extra_size),
            usage: self.usage,
            mapped_at_creation: true,
        });
        {
            let mut buffer_view = temp.slice(..).get_mapped_range_mut();
            let buffer_view_2 = &mut bytemuck::cast_slice_mut(buffer_view.as_mut())[..length];
            fill(buffer_view_2);
        }

        encoder.copy_buffer_to_buffer(&temp, 0, &self.inner, self.size(), extra_size);
        self.length += length;

        encoder.finish()
    }

    #[inline]
    pub fn as_entire_binding(&self) -> BindingResource<'_> {
        self.inner.as_entire_binding()
    }
}

fn default_growth(_: u64, required_size: u64, _: bool) -> u64 {
    const MEGABYTE: u64 = 1024 * 1024;
    (required_size + MEGABYTE - 1) & !MEGABYTE
}
