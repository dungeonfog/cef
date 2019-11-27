use cef::events::WindowsKeyCode;
use cef::events::KeyEvent;
use cef::client::render_handler::CursorType;
use cef::drag::DragOperation;
use cef::values::{Rect, Point};
use cef::client::render_handler::ScreenInfo;
use cef::browser_host::PaintElementType;
use std::{
    time::{Duration, Instant},
    ffi::c_void,
    sync::Arc,
};
use cef::{
    app::{App, AppCallbacks},
    browser::{Browser, BrowserSettings},
    browser_host::BrowserHost,
    browser_process_handler::{BrowserProcessHandler, BrowserProcessHandlerCallbacks},
    client::{
        Client, ClientCallbacks,
        life_span_handler::{LifeSpanHandler, LifeSpanHandlerCallbacks},
        render_handler::{RenderHandler, RenderHandlerCallbacks},
    },
    command_line::CommandLine,
    events::{EventFlags, MouseEvent, MouseButtonType},
    settings::{Settings, LogSeverity},
    window::{WindowInfo, RawWindow},
};
use cef_sys::cef_cursor_handle_t;
use winit::{
    event::{Event, WindowEvent, StartCause, MouseButton, ElementState, MouseScrollDelta, KeyboardInput, VirtualKeyCode},
    dpi::LogicalPosition,
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{CursorIcon, Window, WindowBuilder},
};
use parking_lot::Mutex;

pub struct AppCallbacksImpl {
    browser_process_handler: BrowserProcessHandler,
}
pub struct ClientCallbacksImpl {
    life_span_handler: LifeSpanHandler,
    render_handler: RenderHandler,
}
pub struct LifeSpanHandlerImpl {
    proxy: Mutex<EventLoopProxy<CefEvent>>,
}
pub struct BrowserProcessHandlerCallbacksImpl {
    proxy: Mutex<EventLoopProxy<CefEvent>>,
}
pub struct RenderHandlerCallbacksImpl {
    window: Window,
    renderer: Arc<Mutex<Renderer>>,
    popup_rect: Mutex<Option<Rect>>,
}

#[derive(Clone)]
enum CefEvent {
    ScheduleWork(Instant),
    Quit,
}

impl AppCallbacks for AppCallbacksImpl {
    fn on_before_command_line_processing (&self, process_type: Option<&str>, command_line: CommandLine) {
        if process_type == None {
            command_line.append_switch("disable-gpu");
            command_line.append_switch("disable-gpu-compositing");
        }
    }
    fn get_browser_process_handler(&self) -> Option<BrowserProcessHandler> {
        Some(self.browser_process_handler.clone())
    }
}

impl ClientCallbacks for ClientCallbacksImpl {
    fn get_life_span_handler(&self) -> Option<LifeSpanHandler> {
        Some(self.life_span_handler.clone())
    }
    fn get_render_handler(&self) -> Option<RenderHandler> {
        Some(self.render_handler.clone())
    }
}

impl LifeSpanHandlerCallbacks for LifeSpanHandlerImpl {
    fn on_before_close(&self, _browser: Browser) {
        println!("close browser");
        self.proxy.lock().send_event(CefEvent::Quit).unwrap();
    }
}

impl BrowserProcessHandlerCallbacks for BrowserProcessHandlerCallbacksImpl {
    fn on_schedule_message_pump_work(&self, delay_ms: i64) {
        self.proxy.lock().send_event(CefEvent::ScheduleWork(Instant::now() + Duration::from_millis(delay_ms as u64))).ok();
    }
}

impl RenderHandlerCallbacks for RenderHandlerCallbacksImpl {
    fn get_view_rect(&self, _: Browser) -> Rect {
        let inner_size = self.window.inner_size();
        Rect {
            x: 0,
            y: 0,
            width: inner_size.width.round() as i32,
            height: inner_size.height.round() as i32,
        }
    }
    fn on_popup_show(&self, _browser: Browser, show: bool) {
        if !show {
            *self.popup_rect.lock() = None;
        }
    }
    fn get_screen_point(
        &self,
        _browser: Browser,
        point: Point,
    ) -> Option<Point> {
        let screen_pos = self.window.inner_position().unwrap_or(LogicalPosition::new(0.0, 0.0));
        let physical_pos = (LogicalPosition::new(screen_pos.x + point.x as f64, screen_pos.y + point.y as f64)).to_physical(self.window.hidpi_factor());
        Some(Point::new(physical_pos.x as i32, physical_pos.y as i32))
    }
    fn on_popup_size(&self, _: Browser, mut rect: Rect) {
        let window_size: (u32, u32) = self.window.inner_size().into();
        let window_size = (window_size.0 as i32, window_size.1 as i32);
        rect.x = i32::max(rect.x, 0);
        rect.y = i32::max(rect.y, 0);
        rect.x = i32::min(rect.x, window_size.0 - rect.width);
        rect.y = i32::min(rect.y, window_size.1 - rect.height);
        *self.popup_rect.lock() = Some(rect);
    }
    fn get_screen_info(&self, _: Browser) -> Option<ScreenInfo> {
        let inner_size = self.window.inner_size();
        let rect = Rect {
            x: 0,
            y: 0,
            width: inner_size.width.round() as i32,
            height: inner_size.height.round() as i32,
        };

        Some(ScreenInfo {
            device_scale_factor: self.window.hidpi_factor() as f32,
            depth: 32,
            depth_per_component: 8,
            is_monochrome: false,
            rect: rect,
            available_rect: rect,
        })
    }
    fn on_paint(
        &self,
        _: Browser,
        element_type: PaintElementType,
        dirty_rects: &[Rect],
        buffer: &[u8],
        width: i32,
        height: i32,
    ) {
        println!("paint");
        // FIXME: this completely ignores dirty rects for now and only
        // just re-uploads and re-renders everything anew
        assert_eq!(buffer.len(), 4 * (width * height) as usize);

        let mut renderer = self.renderer.lock();

        renderer.set_window_size(width as u32, height as u32);
        renderer.set_blit_texture(&buffer);
        renderer.blit();
    }
    fn on_accelerated_paint(
        &self,
        _browser: Browser,
        _type_: PaintElementType,
        _dirty_rects: &[Rect],
        _shared_handle: *mut c_void
    ) {
        unimplemented!()
    }
    fn on_cursor_change(
        &self,
        _browser: Browser,
        _cursor: cef_cursor_handle_t,
        type_: CursorType
    ) {
        let winit_cursor = match type_ {
            CursorType::MiddlePanning |
            CursorType::EastPanning |
            CursorType::NorthPanning |
            CursorType::NorthEastPanning |
            CursorType::NorthWestPanning |
            CursorType::SouthPanning |
            CursorType::SouthEastPanning |
            CursorType::SouthWestPanning |
            CursorType::WestPanning |
            CursorType::Custom(_) |
            CursorType::Pointer => Some(CursorIcon::Default),
            CursorType::Cross => Some(CursorIcon::Crosshair),
            CursorType::Hand => Some(CursorIcon::Hand),
            CursorType::IBeam => Some(CursorIcon::Text),
            CursorType::Wait => Some(CursorIcon::Wait),
            CursorType::Help => Some(CursorIcon::Help),
            CursorType::EastResize => Some(CursorIcon::EResize),
            CursorType::NorthResize => Some(CursorIcon::NResize),
            CursorType::NorthEastResize => Some(CursorIcon::NeResize),
            CursorType::NorthWestResize => Some(CursorIcon::NwResize),
            CursorType::SouthResize => Some(CursorIcon::SResize),
            CursorType::SouthEastResize => Some(CursorIcon::SeResize),
            CursorType::SouthWestResize => Some(CursorIcon::SwResize),
            CursorType::WestResize => Some(CursorIcon::WResize),
            CursorType::NorthSouthResize => Some(CursorIcon::NsResize),
            CursorType::EastWestResize => Some(CursorIcon::EwResize),
            CursorType::NorthEastSouthWestResize => Some(CursorIcon::NeswResize),
            CursorType::NorthWestSouthEastResize => Some(CursorIcon::NwseResize),
            CursorType::ColumnResize => Some(CursorIcon::ColResize),
            CursorType::RowResize => Some(CursorIcon::RowResize),
            CursorType::Move => Some(CursorIcon::Move),
            CursorType::VerticalText => Some(CursorIcon::VerticalText),
            CursorType::Cell => Some(CursorIcon::Cell),
            CursorType::ContextMenu => Some(CursorIcon::ContextMenu),
            CursorType::Alias => Some(CursorIcon::Alias),
            CursorType::Progress => Some(CursorIcon::Progress),
            CursorType::NoDrop => Some(CursorIcon::NoDrop),
            CursorType::Copy => Some(CursorIcon::Copy),
            CursorType::None => None,
            CursorType::NotAllowed => Some(CursorIcon::NotAllowed),
            CursorType::ZoomIn => Some(CursorIcon::ZoomIn),
            CursorType::ZoomOut => Some(CursorIcon::ZoomOut),
            CursorType::Grab => Some(CursorIcon::Grab),
            CursorType::Grabbing => Some(CursorIcon::Grabbing),
        };
        match winit_cursor {
            Some(cursor) => {
                self.window.set_cursor_icon(cursor);
                self.window.set_cursor_visible(true);
            },
            None => self.window.set_cursor_visible(false),
        }
    }
    fn update_drag_cursor(&self, _browser: Browser, _operation: DragOperation) {

    }
}

#[derive(Debug)]
pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    swap_chain: wgpu::SwapChain,

    blit_texture: wgpu::Texture,
    blit_texture_bind_group: wgpu::BindGroup,
    blit_render_pipeline: wgpu::RenderPipeline,

    width: u32,
    height: u32,
}

impl Renderer {
    const OUTPUT_ATTACHMENT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

    pub fn new(window: &Window) -> Renderer {
        static SHADER_BLIT_VERT: &[u32] = vk_shader_macros::include_glsl!("examples/blit.vert");
        static SHADER_BLIT_FRAG: &[u32] = vk_shader_macros::include_glsl!("examples/blit.frag");

        let surface = wgpu::Surface::create(window);
        let adapter = wgpu::Adapter::request(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            backends: wgpu::BackendBit::PRIMARY,
        })
        .expect("Failed to find adapter satisfying the options"); // FIXME: handle gracefully
        let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
            extensions: wgpu::Extensions {
                anisotropic_filtering: false,
            },
            limits: wgpu::Limits::default(),
        });

        let window_size = window.inner_size().to_physical(window.hidpi_factor());
        let (width, height) = (window_size.width as u32, window_size.height as u32);

        let swap_chain = Self::create_swap_chain(&device, &surface, width, height);

        let vs_module = device.create_shader_module(&SHADER_BLIT_VERT);
        let fs_module = device.create_shader_module(&SHADER_BLIT_FRAG);

        let blit_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,

            // FIXME: Linear? but we do want to see if there is a size mismatch...
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,

            lod_min_clamp: -100.0,
            lod_max_clamp: 100.0,
            compare_function: wgpu::CompareFunction::Always,
        });

        let blit_texture = device.create_texture(&wgpu::TextureDescriptor {
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
        });
        let blit_texture_view = blit_texture.create_default_view();

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

        let blit_texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &blit_texture_bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&blit_texture_view),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&blit_sampler),
                },
            ],
        });

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
            rasterization_state: None, // FIXME: is there something we need to set here?
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: Self::OUTPUT_ATTACHMENT_FORMAT,

                // FIXME: alpha blending plz
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,

                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            index_format: wgpu::IndexFormat::Uint32,
            vertex_buffers: &[wgpu::VertexBufferDescriptor {
                // FIXME: there are currently no vertex buffers, the
                // fullscreen triangle is generated on the vertex
                // shader
                stride: 0, // mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::InputStepMode::Vertex,
                attributes: &[
                    // wgpu::VertexAttributeDescriptor {
                    //     offset: 0,
                    //     format: wgpu::VertexFormat::Float4,
                    //     shader_location: 0,
                    // },
                    // wgpu::VertexAttributeDescriptor {
                    //     offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    //     format: wgpu::VertexFormat::Float2,
                    //     shader_location: 1,
                    // },
                ],
            }],
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Renderer {
            surface,
            device,
            queue,
            swap_chain,

            blit_texture,
            blit_texture_bind_group,
            blit_render_pipeline,

            width,
            height,
        }
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) {
        // only re-initialize the swapchain if we really have to
        if (width, height) != (self.width, self.height) {
            self.width = width;
            self.height = height;
            self.swap_chain = Self::create_swap_chain(&self.device, &self.surface, width, height);
        }
    }

    pub fn set_blit_texture(&mut self, data: &[u8]) {
        assert_eq!(data.len() % 4, 0);
        Self::upload_texture_bgra8_unorm(
            &self.device,
            &mut self.queue,
            &self.blit_texture,
            self.width,
            self.height,
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
                // FIXME: Do we want vsync?
                present_mode: wgpu::PresentMode::NoVsync,
            },
        )
    }

    fn upload_texture_bgra8_unorm(
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

fn main() {
    match cef::process_type() {
        cef::ProcessType::Renderer |
        cef::ProcessType::Gpu |
        cef::ProcessType::Utility |
        cef::ProcessType::Other => {
            let result = cef::execute_process(None, None);
            if result >= 0 {
                std::process::exit(result);
            }
        }

        cef::ProcessType::Browser => {
            let event_loop: EventLoop<CefEvent> = EventLoop::with_user_event();
            let app = App::new(AppCallbacksImpl {
                browser_process_handler: BrowserProcessHandler::new(BrowserProcessHandlerCallbacksImpl {
                    proxy: Mutex::new(event_loop.create_proxy()),
                })
            });

            #[cfg(not(target_os = "macos"))]
            let settings = Settings::new("./Resources")
                .log_severity(LogSeverity::Verbose)
                .windowless_rendering_enabled(true)
                .external_message_pump(true);
            #[cfg(target_os = "macos")]
            let settings = Settings::new("./Resources")
                .log_severity(LogSeverity::Verbose)
                .windowless_rendering_enabled(true)
                .external_message_pump(true)
                .framework_dir_path("/Library/Frameworks/Chromium Embedded Framework.framework");

            let context = cef::Context::initialize(&settings, Some(app), None).unwrap();

            let window = WindowBuilder::new()
                .with_title("CEF Example Window")
                .build(&event_loop)
                .unwrap();

            let width = window.inner_size().to_physical(window.hidpi_factor()).width.round() as u32;
            let height = window.inner_size().to_physical(window.hidpi_factor()).height.round() as u32;

            let window_info = WindowInfo {
                windowless_rendering_enabled: true,
                parent_window: Some(unsafe{ RawWindow::from_window(&window) }),
                width: width as _,
                height: height as _,
                ..WindowInfo::new()
            };

            let browser_settings = BrowserSettings::new();
            let renderer = Arc::new(Mutex::new(Renderer::new(&window)));

            let client = Client::new(ClientCallbacksImpl {
                life_span_handler: LifeSpanHandler::new(LifeSpanHandlerImpl {
                    proxy: Mutex::new(event_loop.create_proxy()),
                }),
                render_handler: RenderHandler::new(RenderHandlerCallbacksImpl {
                    renderer: renderer.clone(),
                    window,
                    popup_rect: Mutex::new(None),
                })
            });

            let browser = BrowserHost::create_browser_sync(
                &window_info,
                client,
                "https://www.google.com",
                &browser_settings,
                None,
                None,
                &context,
            );

            println!("initialize done");

            let mut mouse_event = MouseEvent {
                x: 0,
                y: 0,
                modifiers: EventFlags::empty(),
            };
            event_loop.run(move |event, _, control_flow| {
                match event {
                    Event::NewEvents(StartCause::ResumeTimeReached{..}) => {
                        *control_flow = ControlFlow::Wait;
                        context.do_message_loop_work();
                    }
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        ..
                    } => {
                        *control_flow = ControlFlow::Exit;
                    }
                    Event::WindowEvent {
                        event,
                        window_id: _,
                    } => {
                        match event {
                            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                            WindowEvent::RedrawRequested => {
                                browser.get_host().invalidate(PaintElementType::View);
                            },
                            WindowEvent::Resized(_) => {
                                browser.get_host().was_resized();
                            },
                            WindowEvent::CursorMoved{position, modifiers, ..} => {
                                mouse_event.modifiers.set(EventFlags::SHIFT_DOWN, modifiers.shift);
                                mouse_event.modifiers.set(EventFlags::CONTROL_DOWN, modifiers.ctrl);
                                mouse_event.modifiers.set(EventFlags::ALT_DOWN, modifiers.alt);
                                mouse_event.x = position.x.round() as _;
                                mouse_event.y = position.y.round() as _;
                                browser.get_host().send_mouse_move_event(
                                    &mouse_event,
                                    false,
                                );
                            },
                            WindowEvent::MouseWheel{delta, ..} => {
                                let (delta_x, delta_y) = match delta {
                                    MouseScrollDelta::LineDelta(x, y) => (20 * x as i32, 20 * y as i32),
                                    MouseScrollDelta::PixelDelta(delta) => (delta.x as _, delta.y as _),
                                };
                                browser.get_host().send_mouse_wheel_event(
                                    &mouse_event,
                                    delta_x,
                                    delta_y,
                                );
                            },
                            WindowEvent::MouseInput{state, button, modifiers, ..} => {
                                mouse_event.modifiers.set(EventFlags::SHIFT_DOWN, modifiers.shift);
                                mouse_event.modifiers.set(EventFlags::CONTROL_DOWN, modifiers.ctrl);
                                mouse_event.modifiers.set(EventFlags::ALT_DOWN, modifiers.alt);
                                let button = match button {
                                    MouseButton::Left => Some(MouseButtonType::Left),
                                    MouseButton::Middle => Some(MouseButtonType::Middle),
                                    MouseButton::Right => Some(MouseButtonType::Right),
                                    _ => None,
                                };
                                let released = match state {
                                    ElementState::Pressed => false,
                                    ElementState::Released => true,
                                };
                                if let Some(button) = button {
                                    match button {
                                        MouseButtonType::Left => mouse_event.modifiers.set(EventFlags::LEFT_MOUSE_BUTTON, !released),
                                        MouseButtonType::Middle => mouse_event.modifiers.set(EventFlags::MIDDLE_MOUSE_BUTTON, !released),
                                        MouseButtonType::Right => mouse_event.modifiers.set(EventFlags::RIGHT_MOUSE_BUTTON, !released),
                                    }
                                    browser.get_host().send_mouse_click_event(
                                        &mouse_event,
                                        button,
                                        released,
                                        1
                                    );
                                }
                            },
                            WindowEvent::KeyboardInput{input: KeyboardInput {state, virtual_keycode, modifiers, ..}, ..} => {
                                mouse_event.modifiers.set(EventFlags::SHIFT_DOWN, modifiers.shift);
                                mouse_event.modifiers.set(EventFlags::CONTROL_DOWN, modifiers.ctrl);
                                mouse_event.modifiers.set(EventFlags::ALT_DOWN, modifiers.alt);
                                if let Some(keycode) = virtual_keycode.and_then(winit_keycode_to_windows_keycode) {
                                    browser.get_host().send_key_event(
                                        match state {
                                            ElementState::Pressed => KeyEvent::KeyDown {
                                                modifiers: mouse_event.modifiers,
                                                windows_key_code: keycode,
                                                is_system_key: false,
                                                focus_on_editable_field: false,
                                            },
                                            ElementState::Released => KeyEvent::KeyUp {
                                                modifiers: mouse_event.modifiers,
                                                windows_key_code: keycode,
                                                is_system_key: false,
                                                focus_on_editable_field: false,
                                            },
                                        }
                                    );
                                }
                            },
                            WindowEvent::ReceivedCharacter(char) => {
                                browser.get_host().send_key_event(
                                    KeyEvent::Char {
                                        modifiers: mouse_event.modifiers,
                                        char,
                                    }
                                );
                            }
                            _ => (),
                        }
                    },
                    Event::UserEvent(event) => match event {
                        CefEvent::ScheduleWork(instant) => {
                            if instant <= Instant::now() {
                                context.do_message_loop_work();
                            } else {
                                *control_flow = ControlFlow::WaitUntil(instant);
                            }
                        },
                        CefEvent::Quit => {
                            context.quit_message_loop();
                        }
                    }
                    _ => (),//*control_flow = ControlFlow::Wait,
                }
            });
        }
    }
}

fn winit_keycode_to_windows_keycode(winit_keycode: VirtualKeyCode) -> Option<WindowsKeyCode> {
    match winit_keycode {
        VirtualKeyCode::Key1 => Some(WindowsKeyCode::Key1),
        VirtualKeyCode::Key2 => Some(WindowsKeyCode::Key2),
        VirtualKeyCode::Key3 => Some(WindowsKeyCode::Key3),
        VirtualKeyCode::Key4 => Some(WindowsKeyCode::Key4),
        VirtualKeyCode::Key5 => Some(WindowsKeyCode::Key5),
        VirtualKeyCode::Key6 => Some(WindowsKeyCode::Key6),
        VirtualKeyCode::Key7 => Some(WindowsKeyCode::Key7),
        VirtualKeyCode::Key8 => Some(WindowsKeyCode::Key8),
        VirtualKeyCode::Key9 => Some(WindowsKeyCode::Key9),
        VirtualKeyCode::Key0 => Some(WindowsKeyCode::Key0),
        VirtualKeyCode::A => Some(WindowsKeyCode::A),
        VirtualKeyCode::B => Some(WindowsKeyCode::B),
        VirtualKeyCode::C => Some(WindowsKeyCode::C),
        VirtualKeyCode::D => Some(WindowsKeyCode::D),
        VirtualKeyCode::E => Some(WindowsKeyCode::E),
        VirtualKeyCode::F => Some(WindowsKeyCode::F),
        VirtualKeyCode::G => Some(WindowsKeyCode::G),
        VirtualKeyCode::H => Some(WindowsKeyCode::H),
        VirtualKeyCode::I => Some(WindowsKeyCode::I),
        VirtualKeyCode::J => Some(WindowsKeyCode::J),
        VirtualKeyCode::K => Some(WindowsKeyCode::K),
        VirtualKeyCode::L => Some(WindowsKeyCode::L),
        VirtualKeyCode::M => Some(WindowsKeyCode::M),
        VirtualKeyCode::N => Some(WindowsKeyCode::N),
        VirtualKeyCode::O => Some(WindowsKeyCode::O),
        VirtualKeyCode::P => Some(WindowsKeyCode::P),
        VirtualKeyCode::Q => Some(WindowsKeyCode::Q),
        VirtualKeyCode::R => Some(WindowsKeyCode::R),
        VirtualKeyCode::S => Some(WindowsKeyCode::S),
        VirtualKeyCode::T => Some(WindowsKeyCode::T),
        VirtualKeyCode::U => Some(WindowsKeyCode::U),
        VirtualKeyCode::V => Some(WindowsKeyCode::V),
        VirtualKeyCode::W => Some(WindowsKeyCode::W),
        VirtualKeyCode::X => Some(WindowsKeyCode::X),
        VirtualKeyCode::Y => Some(WindowsKeyCode::Y),
        VirtualKeyCode::Z => Some(WindowsKeyCode::Z),
        VirtualKeyCode::Escape => Some(WindowsKeyCode::Escape),
        VirtualKeyCode::F1 => Some(WindowsKeyCode::F1),
        VirtualKeyCode::F2 => Some(WindowsKeyCode::F2),
        VirtualKeyCode::F3 => Some(WindowsKeyCode::F3),
        VirtualKeyCode::F4 => Some(WindowsKeyCode::F4),
        VirtualKeyCode::F5 => Some(WindowsKeyCode::F5),
        VirtualKeyCode::F6 => Some(WindowsKeyCode::F6),
        VirtualKeyCode::F7 => Some(WindowsKeyCode::F7),
        VirtualKeyCode::F8 => Some(WindowsKeyCode::F8),
        VirtualKeyCode::F9 => Some(WindowsKeyCode::F9),
        VirtualKeyCode::F10 => Some(WindowsKeyCode::F10),
        VirtualKeyCode::F11 => Some(WindowsKeyCode::F11),
        VirtualKeyCode::F12 => Some(WindowsKeyCode::F12),
        VirtualKeyCode::F13 => Some(WindowsKeyCode::F13),
        VirtualKeyCode::F14 => Some(WindowsKeyCode::F14),
        VirtualKeyCode::F15 => Some(WindowsKeyCode::F15),
        VirtualKeyCode::F16 => Some(WindowsKeyCode::F16),
        VirtualKeyCode::F17 => Some(WindowsKeyCode::F17),
        VirtualKeyCode::F18 => Some(WindowsKeyCode::F18),
        VirtualKeyCode::F19 => Some(WindowsKeyCode::F19),
        VirtualKeyCode::F20 => Some(WindowsKeyCode::F20),
        VirtualKeyCode::F21 => Some(WindowsKeyCode::F21),
        VirtualKeyCode::F22 => Some(WindowsKeyCode::F22),
        VirtualKeyCode::F23 => Some(WindowsKeyCode::F23),
        VirtualKeyCode::F24 => Some(WindowsKeyCode::F24),
        VirtualKeyCode::Snapshot => Some(WindowsKeyCode::Snapshot),
        VirtualKeyCode::Scroll => Some(WindowsKeyCode::Scroll),
        VirtualKeyCode::Pause => Some(WindowsKeyCode::Pause),
        VirtualKeyCode::Insert => Some(WindowsKeyCode::Insert),
        VirtualKeyCode::Home => Some(WindowsKeyCode::Home),
        VirtualKeyCode::Delete => Some(WindowsKeyCode::Delete),
        VirtualKeyCode::End => Some(WindowsKeyCode::End),
        VirtualKeyCode::PageDown => Some(WindowsKeyCode::Next),
        VirtualKeyCode::PageUp => Some(WindowsKeyCode::Prior),
        VirtualKeyCode::Left => Some(WindowsKeyCode::Left),
        VirtualKeyCode::Up => Some(WindowsKeyCode::Up),
        VirtualKeyCode::Right => Some(WindowsKeyCode::Right),
        VirtualKeyCode::Down => Some(WindowsKeyCode::Down),
        VirtualKeyCode::Back => Some(WindowsKeyCode::Back),
        VirtualKeyCode::Return => Some(WindowsKeyCode::Return),
        VirtualKeyCode::Space => Some(WindowsKeyCode::Space),
        VirtualKeyCode::Numlock => Some(WindowsKeyCode::Numlock),
        VirtualKeyCode::Numpad0 => Some(WindowsKeyCode::Numpad0),
        VirtualKeyCode::Numpad1 => Some(WindowsKeyCode::Numpad1),
        VirtualKeyCode::Numpad2 => Some(WindowsKeyCode::Numpad2),
        VirtualKeyCode::Numpad3 => Some(WindowsKeyCode::Numpad3),
        VirtualKeyCode::Numpad4 => Some(WindowsKeyCode::Numpad4),
        VirtualKeyCode::Numpad5 => Some(WindowsKeyCode::Numpad5),
        VirtualKeyCode::Numpad6 => Some(WindowsKeyCode::Numpad6),
        VirtualKeyCode::Numpad7 => Some(WindowsKeyCode::Numpad7),
        VirtualKeyCode::Numpad8 => Some(WindowsKeyCode::Numpad8),
        VirtualKeyCode::Numpad9 => Some(WindowsKeyCode::Numpad9),
        VirtualKeyCode::Add => Some(WindowsKeyCode::Add),
        VirtualKeyCode::Apps => Some(WindowsKeyCode::Apps),
        VirtualKeyCode::Capital => Some(WindowsKeyCode::Capital),
        VirtualKeyCode::Comma => Some(WindowsKeyCode::OemComma),
        VirtualKeyCode::Convert => Some(WindowsKeyCode::Convert),
        VirtualKeyCode::Decimal => Some(WindowsKeyCode::Decimal),
        VirtualKeyCode::Divide => Some(WindowsKeyCode::Divide),
        VirtualKeyCode::Equals => Some(WindowsKeyCode::OemPlus),
        VirtualKeyCode::Kana => Some(WindowsKeyCode::Kana),
        VirtualKeyCode::Kanji => Some(WindowsKeyCode::Kanji),
        VirtualKeyCode::LAlt => Some(WindowsKeyCode::LMenu),
        VirtualKeyCode::LControl => Some(WindowsKeyCode::LControl),
        VirtualKeyCode::LShift => Some(WindowsKeyCode::LShift),
        VirtualKeyCode::LWin => Some(WindowsKeyCode::LWin),
        VirtualKeyCode::Mail => Some(WindowsKeyCode::LaunchMail),
        VirtualKeyCode::MediaSelect => Some(WindowsKeyCode::LaunchMediaSelect),
        VirtualKeyCode::MediaStop => Some(WindowsKeyCode::MediaStop),
        VirtualKeyCode::Minus => Some(WindowsKeyCode::OemMinus),
        VirtualKeyCode::Multiply => Some(WindowsKeyCode::Multiply),
        VirtualKeyCode::Mute => Some(WindowsKeyCode::VolumeMute),
        VirtualKeyCode::NavigateForward => Some(WindowsKeyCode::BrowserForward),
        VirtualKeyCode::NavigateBackward => Some(WindowsKeyCode::BrowserBack),
        VirtualKeyCode::NextTrack => Some(WindowsKeyCode::MediaNextTrack),
        VirtualKeyCode::NoConvert => Some(WindowsKeyCode::NonConvert),
        VirtualKeyCode::OEM102 => Some(WindowsKeyCode::Oem102),
        VirtualKeyCode::Period => Some(WindowsKeyCode::OemPeriod),
        VirtualKeyCode::PlayPause => Some(WindowsKeyCode::MediaPlayPause),
        VirtualKeyCode::PrevTrack => Some(WindowsKeyCode::MediaPrevTrack),
        VirtualKeyCode::RAlt => Some(WindowsKeyCode::RMenu),
        VirtualKeyCode::RControl => Some(WindowsKeyCode::RControl),
        VirtualKeyCode::RShift => Some(WindowsKeyCode::RShift),
        VirtualKeyCode::RWin => Some(WindowsKeyCode::RWin),
        VirtualKeyCode::Sleep => Some(WindowsKeyCode::Sleep),
        VirtualKeyCode::Subtract => Some(WindowsKeyCode::Subtract),
        VirtualKeyCode::Tab => Some(WindowsKeyCode::Tab),
        VirtualKeyCode::VolumeDown => Some(WindowsKeyCode::VolumeDown),
        VirtualKeyCode::VolumeUp => Some(WindowsKeyCode::VolumeUp),
        VirtualKeyCode::WebFavorites => Some(WindowsKeyCode::BrowserFavorites),
        VirtualKeyCode::WebForward => Some(WindowsKeyCode::BrowserForward),
        VirtualKeyCode::WebHome => Some(WindowsKeyCode::BrowserHome),
        VirtualKeyCode::WebRefresh => Some(WindowsKeyCode::BrowserRefresh),
        VirtualKeyCode::WebSearch => Some(WindowsKeyCode::BrowserSearch),
        VirtualKeyCode::WebStop => Some(WindowsKeyCode::BrowserStop),
        // These key-codes should be unmapped by reversing MapVirtualKeyA.
        // VirtualKeyCode::Apostrophe => Some(WindowsKeyCode::Apostrophe),
        // VirtualKeyCode::Backslash => Some(WindowsKeyCode::Backslash),
        // VirtualKeyCode::Grave => Some(WindowsKeyCode::Grave),
        // VirtualKeyCode::LBracket => Some(WindowsKeyCode::LBracket),
        // VirtualKeyCode::RBracket => Some(WindowsKeyCode::RBracket),
        // VirtualKeyCode::Semicolon => Some(WindowsKeyCode::Semicolon),
        // VirtualKeyCode::Slash => Some(WindowsKeyCode::Slash),
        _ => None,
    }
}
