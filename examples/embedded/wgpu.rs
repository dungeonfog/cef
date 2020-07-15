use crate::Renderer;
use winit::{
    window::{Window, WindowBuilder},
    event_loop::EventLoop,
};
use cef::{
    browser_host::PaintElementType,
    values::Rect,
};

#[derive(Debug)]
pub struct WgpuRenderer {
    window: Window,

    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,

    blit_sampler: wgpu::Sampler,
    blit_texture: wgpu::Texture,
    blit_texture_bind_group_layout: wgpu::BindGroupLayout,
    blit_texture_bind_group: wgpu::BindGroup,
    blit_render_pipeline: wgpu::RenderPipeline,
    blit_texture_width: u32,
    blit_texture_height: u32,

    swap_chain_width: u32,
    swap_chain_height: u32,
}

impl Renderer for WgpuRenderer {
    fn new<T>(window_builder: WindowBuilder, el: &EventLoop<T>) -> Self {
        WgpuRenderer::new(window_builder.build(el).unwrap())
    }
    fn window(&self) -> &Window {
        &self.window
    }

    fn on_paint(
        &mut self,
        _: PaintElementType,
        _: &[Rect],
        buffer: &[u8],
        width: i32,
        height: i32,
    ) {
        // FIXME: this completely ignores dirty rects for now and only
        // just re-uploads and re-renders everything anew
        assert_eq!(buffer.len(), 4 * (width * height) as usize);

        self.set_window_size(
            width as u32,
            height as u32,
        );
        self.set_blit_texture(width as u32, height as u32, buffer);
        self.blit();
    }
    fn set_popup_rect(&mut self, _rect: Option<Rect>) {}
}

impl WgpuRenderer {
    const OUTPUT_ATTACHMENT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

    fn new(window: Window) -> WgpuRenderer {
        static SHADER_BLIT_VERT: &[u32] = vk_shader_macros::include_glsl!("blit.vert");
        static SHADER_BLIT_FRAG: &[u32] = vk_shader_macros::include_glsl!("blit.frag");

        let surface = wgpu::Surface::create(&window);
        let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            backends: wgpu::BackendBit::PRIMARY,
        })
        .expect("Failed to find adapter satisfying the options");
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        });

        let window_size = window.inner_size();
        let (swap_chain_width, swap_chain_height) =
            (window_size.width as u32, window_size.height as u32);

        let swap_chain =
            Self::create_swap_chain(&device, &surface, swap_chain_width, swap_chain_height);

        let vs_module = device.create_shader_module(&SHADER_BLIT_VERT);
        let fs_module = device.create_shader_module(&SHADER_BLIT_FRAG);

        let blit_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,

            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,

            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        });

        let blit_texture_width = swap_chain_width;
        let blit_texture_height = swap_chain_height;
        let blit_texture = Self::create_texture(&device, blit_texture_width, blit_texture_height);

        let blit_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                bindings: &[
                    wgpu::BindGroupLayoutBinding {
                        binding: 0,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::SampledTexture {
                            multisampled: false,
                            dimension: wgpu::TextureViewDimension::D2,
                        },
                    },
                    wgpu::BindGroupLayoutBinding {
                        binding: 1,
                        visibility: wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::Sampler,
                    },
                ],
            });

        let blit_texture_bind_group = Self::create_bind_group(
            &device,
            &blit_texture_bind_group_layout,
            &blit_texture,
            &blit_sampler,
        );

        let blit_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                bind_group_layouts: &[&blit_texture_bind_group_layout],
            });

        let blit_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &blit_render_pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: None,
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: Self::OUTPUT_ATTACHMENT_FORMAT,

                // FIXME: alpha blending
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,

                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                stride: 0,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[],
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        WgpuRenderer {
            window,

            surface,
            device,
            queue,
            swap_chain,

            blit_sampler,
            blit_texture,
            blit_texture_bind_group_layout,
            blit_texture_bind_group,
            blit_render_pipeline,
            blit_texture_width,
            blit_texture_height,

            swap_chain_width,
            swap_chain_height,
        }
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) {
        // Only re-initialize the swapchain if we really have to
        if (width, height) != (self.swap_chain_width, self.swap_chain_height) {
            self.swap_chain = Self::create_swap_chain(&self.device, &self.surface, width, height);

            self.swap_chain_width = width;
            self.swap_chain_height = height;
        }
    }

    pub fn set_blit_texture(&mut self, width: u32, height: u32, data: &[u8]) {
        assert_eq!(data.len() % 4, 0);

        // Re-initialize the resources if needed
        if (width, height) != (self.blit_texture_width, self.blit_texture_height) {
            self.blit_texture = Self::create_texture(&self.device, width, height);
            self.blit_texture_bind_group = Self::create_bind_group(
                &self.device,
                &self.blit_texture_bind_group_layout,
                &self.blit_texture,
                &self.blit_sampler,
            );

            self.blit_texture_width = width;
            self.blit_texture_height = height;
        }

        Self::upload_texture(
            &self.device,
            &mut self.queue,
            &self.blit_texture,
            self.blit_texture_width,
            self.blit_texture_height,
            data,
        );
    }

    pub fn blit(&mut self) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        let frame = self.swap_chain.get_next_texture();

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.0,
                        g: 0.2,
                        b: 0.5,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.blit_render_pipeline);
            rpass.set_bind_group(0, &self.blit_texture_bind_group, &[]);

            rpass.draw(0..3, 0..1);
        }

        self.queue.submit(&[encoder.finish()]);
    }

    fn create_swap_chain(
        device: &wgpu::Device,
        surface: &wgpu::Surface,
        width: u32,
        height: u32,
    ) -> wgpu::SwapChain {
        device.create_swap_chain(
            &surface,
            &wgpu::SwapChainDescriptor {
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
                format: Self::OUTPUT_ATTACHMENT_FORMAT,
                width,
                height,
                present_mode: wgpu::PresentMode::Vsync,
            },
        )
    }

    fn create_texture(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            size: wgpu::Extent3d {
                width,
                height,
                depth: 1,
            },
            array_layer_count: 1,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        })
    }

    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        texture: &wgpu::Texture,
        sampler: &wgpu::Sampler,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.create_default_view()),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        })
    }

    fn upload_texture(
        device: &wgpu::Device,
        queue: &mut wgpu::Queue,
        texture: &wgpu::Texture,
        width: u32,
        height: u32,
        data: &[u8],
    ) {
        let transfer_buffer = device
            .create_buffer_mapped(data.len(), wgpu::BufferUsage::COPY_SRC)
            .fill_from_slice(data);

        let byte_count = data.len() as u32;
        let pixel_size = byte_count / width / height;

        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { todo: 0 });
        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &transfer_buffer,
                offset: 0,
                row_pitch: pixel_size * width,
                image_height: height,
            },
            wgpu::TextureCopyView {
                texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth: 1,
            },
        );

        queue.submit(&[encoder.finish()]);
    }
}
