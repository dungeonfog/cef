#[cfg(feature = "winit-blit-renderer")]
mod winit_blit;
#[cfg(feature = "gullery-renderer")]
mod gullery;
#[cfg(feature = "wgpu-renderer")]
mod wgpu;
#[cfg(not(any(feature = "winit-blit-renderer", feature = "gullery-renderer", feature = "wgpu-renderer")))]
compile_error!("At least one renderer feature must be enabled! Enable winit-blit-renderer, gullery-renderer, or wgpu-renderer to continue");

use cef::color::Color;
use cef::browser_host::PaintElementType;
use cef::client::render_handler::CursorType;
use cef::client::render_handler::ScreenInfo;
use cef::drag::DragOperation;
use cef::events::KeyEvent;
use cef::events::WindowsKeyCode;
use cef::events::{PointerType, TouchEvent, TouchEventType};
use cef::values::{Point, Rect};
use cef::{
    app::{App, AppCallbacks},
    browser::{Browser, BrowserSettings},
    browser_host::BrowserHost,
    browser_process_handler::{BrowserProcessHandler, BrowserProcessHandlerCallbacks},
    client::{
        life_span_handler::{LifeSpanHandler, LifeSpanHandlerCallbacks},
        render_handler::{RenderHandler, RenderHandlerCallbacks},
        Client, ClientCallbacks,
    },
    command_line::CommandLine,
    events::{EventFlags, MouseButtonType, MouseEvent},
    settings::{LogSeverity, Settings},
    window::{RawWindow, WindowInfo},
};
use cef_sys::cef_cursor_handle_t;
use parking_lot::Mutex;
use std::{
    ffi::c_void,
    sync::Arc,
    time::{Duration, Instant},
};
use winit::event::{Touch, TouchPhase};
use winit::{
    dpi::{LogicalPosition, PhysicalPosition, PhysicalSize},
    event::{
        ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, StartCause,
        VirtualKeyCode, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{CursorIcon, Window, WindowBuilder, CustomCursorIcon},
};

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
pub struct RenderHandlerCallbacksImpl<R: Renderer> {
    renderer: Arc<Mutex<R>>,
}

pub trait Renderer: 'static + Send {
    fn new<T>(window_builder: WindowBuilder, el: &EventLoop<T>) -> Self;
    fn window(&self) -> &Window;
    fn on_paint(
        &mut self,
        element_type: PaintElementType,
        dirty_rects: &[Rect],
        buffer: &[u8],
        width: i32,
        height: i32,
    );
    fn set_popup_rect(&mut self, rect: Option<Rect>);
}

#[derive(Debug, Clone)]
enum CefEvent {
    ScheduleWork(Instant),
    Quit,
}

impl AppCallbacks for AppCallbacksImpl {
    fn on_before_command_line_processing(
        &self,
        process_type: Option<&str>,
        command_line: CommandLine,
    ) {
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
        self.proxy.lock().send_event(CefEvent::Quit).unwrap();
    }
}

impl BrowserProcessHandlerCallbacks for BrowserProcessHandlerCallbacksImpl {
    fn on_schedule_message_pump_work(&self, delay_ms: i64) {
        self.proxy
            .lock()
            .send_event(CefEvent::ScheduleWork(
                Instant::now() + Duration::from_millis(delay_ms as u64),
            ))
            .ok();
    }
}

impl<R: Renderer> RenderHandlerCallbacks for RenderHandlerCallbacksImpl<R> {
    fn get_view_rect(&self, _: Browser) -> Rect {
        let renderer = self.renderer.lock();
        let window = renderer.window();
        let inner_size = window.inner_size().to_logical::<i32>(window.scale_factor());
        Rect {
            x: 0,
            y: 0,
            width: inner_size.width,
            height: inner_size.height,
        }
    }
    fn on_popup_show(&self, _browser: Browser, show: bool) {
        if !show {
            self.renderer.lock().set_popup_rect(None);
        }
    }
    fn get_screen_point(
        &self,
        _browser: Browser,
        point: Point,
    ) -> Option<Point> {
        let renderer = self.renderer.lock();
        let window = renderer.window();

        let screen_pos = window.inner_position().unwrap_or(PhysicalPosition::new(0, 0));
        let point_physical = LogicalPosition::new(point.x, point.y).to_physical::<i32>(window.scale_factor());
        Some(Point::new(screen_pos.x + point_physical.x, screen_pos.y + point_physical.y))
    }
    fn on_popup_size(&self, _: Browser, mut rect: Rect) {
        let mut renderer = self.renderer.lock();
        let window = renderer.window();

        let window_size: (u32, u32) = window.inner_size().into();
        let window_size = (window_size.0 as i32, window_size.1 as i32);
        rect.x = i32::max(rect.x, 0);
        rect.y = i32::max(rect.y, 0);
        rect.x = i32::min(rect.x, window_size.0 - rect.width);
        rect.y = i32::min(rect.y, window_size.1 - rect.height);
        renderer.set_popup_rect(Some(rect));
    }
    fn get_screen_info(&self, _: Browser) -> Option<ScreenInfo> {
        let renderer = self.renderer.lock();
        let window = renderer.window();

        let inner_size = window.inner_size().to_logical::<i32>(window.scale_factor());
        let rect = Rect {
            x: 0,
            y: 0,
            width: inner_size.width,
            height: inner_size.height,
        };

        Some(ScreenInfo {
            device_scale_factor: window.scale_factor() as f32,
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
        // FIXME: this completely ignores dirty rects for now and only
        // just re-uploads and re-renders everything anew
        assert_eq!(buffer.len(), 4 * (width * height) as usize);

        let mut renderer = self.renderer.lock();
        renderer.on_paint(element_type, dirty_rects, buffer, width, height);
    }
    fn on_accelerated_paint(
        &self,
        _browser: Browser,
        _type_: PaintElementType,
        _dirty_rects: &[Rect],
        _shared_handle: *mut c_void,
    ) {
        unimplemented!()
    }
    fn on_cursor_change(&self, _browser: Browser, _cursor: cef_cursor_handle_t, type_: CursorType) {
        // this is a good website for testing cursor changes
        // http://html5advent2011.digitpaint.nl/3/index.html
        let winit_cursor = match type_ {
            CursorType::MiddlePanning
            | CursorType::EastPanning
            | CursorType::NorthPanning
            | CursorType::NorthEastPanning
            | CursorType::NorthWestPanning
            | CursorType::SouthPanning
            | CursorType::SouthEastPanning
            | CursorType::SouthWestPanning
            | CursorType::WestPanning
            | CursorType::MiddlePanning
            | CursorType::MiddlePanningVertical
            | CursorType::MiddlePanningHorizontal
            | CursorType::DndNone
            | CursorType::DndMove
            | CursorType::DndCopy
            | CursorType::DndLink
            | CursorType::Pointer => Some(CursorIcon::Default),
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
            CursorType::Custom(custom_cursor) => {
                let hot_spot = PhysicalPosition::new(custom_cursor.hotspot.x as u32, custom_cursor.hotspot.y as u32);
                let size = PhysicalSize::new(custom_cursor.size.width as u32, custom_cursor.size.height as u32);
                let mut buffer = custom_cursor.buffer.to_owned();
                for pixel in buffer.chunks_mut(4) {
                    let (l, r) = pixel.split_at_mut(2);
                    std::mem::swap(&mut l[0], &mut r[0]);
                }
                Some(CustomCursorIcon::from_rgba(&buffer, size, hot_spot)
                    .ok().map(|c| CursorIcon::Custom(c))
                    .unwrap_or(CursorIcon::Default))
            }
        };
        let renderer = self.renderer.lock();
        let window = renderer.window();

        match winit_cursor {
            Some(cursor) => {
                window.set_cursor_icon(cursor);
                window.set_cursor_visible(true);
            }
            None => window.set_cursor_visible(false),
        }
    }
    fn update_drag_cursor(&self, _browser: Browser, _operation: DragOperation) {}
}

fn main() {
    match cef::process_type() {
        cef::ProcessType::Renderer
        | cef::ProcessType::Gpu
        | cef::ProcessType::Utility
        | cef::ProcessType::Other => {
            let result = cef::execute_process(None, None);
            if result >= 0 {
                std::process::exit(result);
            }
        }

        cef::ProcessType::Browser => {
            let framework_dir_path = {
                #[cfg(target_os = "macos")] {
                    Some(cef::load_framework(None).unwrap())
                }
                #[cfg(not(target_os = "macos"))] {
                    None
                }
            };
            let logger = cef::logging::Logger::builder()
                .level(log::LevelFilter::Trace)
                .build();
            let logger = Box::new(logger);
            log::set_boxed_logger(logger)
                .map(|()| log::set_max_level(log::LevelFilter::Trace))
                .unwrap();

            log::info!("testing logs");

            let event_loop: EventLoop<CefEvent> = EventLoop::with_user_event();
            let app = App::new(AppCallbacksImpl {
                browser_process_handler: BrowserProcessHandler::new(
                    BrowserProcessHandlerCallbacksImpl {
                        proxy: Mutex::new(event_loop.create_proxy()),
                    },
                ),
            });

            let mut settings = Settings::new()
                .log_severity(LogSeverity::Verbose)
                .windowless_rendering_enabled(true)
                .external_message_pump(true);

            settings.framework_dir_path = framework_dir_path;

            let context = cef::Context::initialize(settings, Some(app), None).unwrap();

            let window_builder = WindowBuilder::new()
                .with_title("CEF Example Window");

            #[cfg(feature = "winit-blit-renderer")]
            let renderer = crate::winit_blit::WinitBlitRenderer::new(window_builder, &event_loop);
            #[cfg(feature = "gullery-renderer")]
            let renderer = crate::gullery::GulleryRenderer::new(window_builder, &event_loop);
            #[cfg(feature = "wgpu-renderer")]
            let renderer = crate::wgpu::WgpuRenderer::new(window_builder, &event_loop);

            let width = renderer.window().inner_size().width;
            let height = renderer.window().inner_size().height;

            let window_info = WindowInfo {
                windowless_rendering_enabled: true,
                parent_window: Some(unsafe { RawWindow::from_window(renderer.window()) }),
                width: width as _,
                height: height as _,
                ..WindowInfo::new()
            };

            let browser_settings = BrowserSettings {
                background_color: Color::rgba(1.0, 1.0, 1.0, 1.0),
                ..BrowserSettings::new()
            };

            let renderer = Arc::new(Mutex::new(renderer));
            let client = Client::new(ClientCallbacksImpl {
                life_span_handler: LifeSpanHandler::new(LifeSpanHandlerImpl {
                    proxy: Mutex::new(event_loop.create_proxy()),
                }),
                render_handler: RenderHandler::new(RenderHandlerCallbacksImpl {
                    renderer: Arc::clone(&renderer),
                }),
            });

            let browser = BrowserHost::create_browser_sync(
                &window_info,
                client,
                "http://html5advent2011.digitpaint.nl/3/index.html",
                &browser_settings,
                None,
                None,
            );

            println!("initialize done");

            let mut mouse_event = MouseEvent {
                x: 0,
                y: 0,
                modifiers: EventFlags::empty(),
            };
            let poll_duration = Duration::new(1, 0) / 30;
            let mut poll_instant = Instant::now() + poll_duration;
            let mut scheduled_work_queue = vec![poll_instant];
            event_loop.run(move |event, _, control_flow| {
                match event {
                    Event::NewEvents(StartCause::ResumeTimeReached{..}) => {
                        while scheduled_work_queue.len() > 0 && scheduled_work_queue[0] <= Instant::now() {
                            let work_time = scheduled_work_queue.remove(0);
                            if work_time == poll_instant {
                                poll_instant = work_time + poll_duration;
                                scheduled_work_queue.push(poll_instant);
                            }
                            context.do_message_loop_work();
                        }
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
                    } => match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(_) => {
                            browser.get_host().was_resized();
                        }
                        WindowEvent::CursorMoved {
                            position,
                            modifiers,
                            ..
                        } => {
                            mouse_event
                                .modifiers
                                .set(EventFlags::SHIFT_DOWN, modifiers.shift());
                            mouse_event
                                .modifiers
                                .set(EventFlags::CONTROL_DOWN, modifiers.ctrl());
                            mouse_event
                                .modifiers
                                .set(EventFlags::ALT_DOWN, modifiers.alt());
                            mouse_event.x = position.x.round() as _;
                            mouse_event.y = position.y.round() as _;
                            browser
                                .get_host()
                                .send_mouse_move_event(&mouse_event, false);
                        }
                        WindowEvent::MouseWheel { delta, .. } => {
                            let (delta_x, delta_y) = match delta {
                                MouseScrollDelta::LineDelta(x, y) => (20 * x as i32, 20 * y as i32),
                                MouseScrollDelta::PixelDelta(delta) => (delta.x as _, delta.y as _),
                            };
                            browser.get_host().send_mouse_wheel_event(
                                &mouse_event,
                                delta_x,
                                delta_y,
                            );
                        }
                        WindowEvent::MouseInput {
                            state,
                            button,
                            modifiers,
                            ..
                        } => {
                            mouse_event
                                .modifiers
                                .set(EventFlags::SHIFT_DOWN, modifiers.shift());
                            mouse_event
                                .modifiers
                                .set(EventFlags::CONTROL_DOWN, modifiers.ctrl());
                            mouse_event
                                .modifiers
                                .set(EventFlags::ALT_DOWN, modifiers.alt());
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
                                    MouseButtonType::Left => mouse_event
                                        .modifiers
                                        .set(EventFlags::LEFT_MOUSE_BUTTON, !released),
                                    MouseButtonType::Middle => mouse_event
                                        .modifiers
                                        .set(EventFlags::MIDDLE_MOUSE_BUTTON, !released),
                                    MouseButtonType::Right => mouse_event
                                        .modifiers
                                        .set(EventFlags::RIGHT_MOUSE_BUTTON, !released),
                                }
                                browser.get_host().send_mouse_click_event(
                                    &mouse_event,
                                    button,
                                    released,
                                    1,
                                );
                            }
                        }
                        WindowEvent::Touch(Touch {
                            phase,
                            location,
                            force,
                            id,
                            ..
                        }) => browser.get_host().send_touch_event(&TouchEvent {
                            touch_id: id as i32,
                            x: location.x as f32,
                            y: location.y as f32,
                            radius_x: 1.0,
                            radius_y: 1.0,
                            rotation_angle: 0.0,
                            pressure: force.map(|f| f.normalized() as f32).unwrap_or(1.0),
                            event_type: match phase {
                                TouchPhase::Started => TouchEventType::Pressed,
                                TouchPhase::Moved => TouchEventType::Moved,
                                TouchPhase::Ended => TouchEventType::Released,
                                TouchPhase::Cancelled => TouchEventType::Cancelled,
                            },
                            modifiers: mouse_event.modifiers,
                            pointer_type: PointerType::Touch,
                        }),
                        WindowEvent::KeyboardInput{input: KeyboardInput {state, virtual_keycode, scancode, modifiers, ..}, ..} => {
                            mouse_event.modifiers.set(EventFlags::SHIFT_DOWN, modifiers.shift());
                            mouse_event.modifiers.set(EventFlags::CONTROL_DOWN, modifiers.ctrl());
                            mouse_event.modifiers.set(EventFlags::ALT_DOWN, modifiers.alt());
                            if let Some(keycode) = virtual_keycode.and_then(winit_keycode_to_windows_keycode) {
                                browser.get_host().send_key_event(
                                    match state {
                                        ElementState::Pressed => KeyEvent::KeyDown {
                                            modifiers: mouse_event.modifiers,
                                            windows_key_code: keycode,
                                            native_key_code: scancode as _,
                                            is_system_key: false,
                                            focus_on_editable_field: false,
                                        },
                                        ElementState::Released => KeyEvent::KeyUp {
                                            modifiers: mouse_event.modifiers,
                                            windows_key_code: keycode,
                                            native_key_code: scancode as _,
                                            is_system_key: false,
                                            focus_on_editable_field: false,
                                        },
                                    }
                                );
                            }
                        },
                        WindowEvent::ReceivedCharacter(char) => {
                            browser.get_host().send_key_event(KeyEvent::Char {
                                modifiers: mouse_event.modifiers,
                                char,
                            });
                        }
                        _ => (),
                    },
                    Event::UserEvent(event) => match event {
                        CefEvent::ScheduleWork(instant) => {
                            if instant <= Instant::now() {
                                context.do_message_loop_work();
                            } else {
                                let i = match scheduled_work_queue.binary_search(&instant) {
                                    Ok(i) | Err(i) => i
                                };
                                scheduled_work_queue.insert(i, instant);
                            }
                        },
                        CefEvent::Quit => {
                            context.quit_message_loop();
                        }
                    },
                    Event::MainEventsCleared => {
                        if scheduled_work_queue.len() > 0 {
                            *control_flow = ControlFlow::WaitUntil(scheduled_work_queue[0]);
                        } else {
                            *control_flow = ControlFlow::Wait;
                        }
                    },
                    Event::RedrawRequested(_) => {
                    },
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
