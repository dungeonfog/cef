use winit::event::{Touch, TouchPhase};
use cef::events::{TouchEvent, TouchEventType, PointerType};
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
use winit_blit::{PixelBufferTyped, BGRA};
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
    pixel_buffer: Mutex<PixelBufferTyped<BGRA>>,
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
        println!("schedule work {}", delay_ms);
        self.proxy.lock().send_event(CefEvent::ScheduleWork(Instant::now() + Duration::from_millis(delay_ms as u64))).ok();
    }
    fn on_before_child_process_launch(&self, command_line: CommandLine) {
        command_line.append_switch("enable-high-dpi-support");
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
        height: i32
    ) {
        println!("paint {:?}", dirty_rects);
        let buffer = BGRA::from_raw_slice(buffer);
        assert_eq!(buffer.len(), (width * height) as usize);
        let buffer_row = |row: usize| {
            &buffer[row as usize * width as usize..(1 + row) as usize * width as usize]
        };
        let mut pixel_buffer = self.pixel_buffer.lock();
        if pixel_buffer.width() != width as u32 || pixel_buffer.height() != height as u32 {
            *pixel_buffer = PixelBufferTyped::new_supported(width as u32, height as u32, &self.window);
        }
        match (element_type, *self.popup_rect.lock()) {
            (PaintElementType::View, _) => {
                for rect in dirty_rects {
                    let row_span = rect.x as usize..rect.x as usize + rect.width as usize;
                    for row in (rect.y..rect.y+rect.height).map(|r| r as usize) {
                        let pixel_buffer_row =
                            &mut pixel_buffer.row_mut(row as u32).unwrap()
                                [row_span.clone()];
                        pixel_buffer_row.copy_from_slice(&buffer_row(row)[row_span.clone()]);
                    }

                    pixel_buffer.blit_rect(
                        (rect.x as u32, rect.y as u32),
                        (rect.x as u32, rect.y as u32),
                        (rect.width as u32, rect.height as u32),
                        &self.window
                    ).unwrap();
                }
            },
            (PaintElementType::Popup, Some(rect)) => {
                let row_span = rect.x as usize..rect.x as usize + rect.width as usize;
                for row in (rect.y..rect.y+rect.height).map(|r| r as usize) {
                    let pixel_buffer_row =
                        &mut pixel_buffer.row_mut(row as u32).unwrap()
                            [row_span.clone()];
                    pixel_buffer_row.copy_from_slice(&buffer_row(row)[row_span.clone()]);
                }

                pixel_buffer.blit_rect(
                    (rect.x as u32, rect.y as u32),
                    (rect.x as u32, rect.y as u32),
                    (rect.width as u32, rect.height as u32),
                    &self.window
                ).unwrap();
            },
            _ => (),
        }
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

            let settings = Settings::new()
                .windowless_rendering_enabled(true)
                .log_severity(LogSeverity::Verbose)
                .external_message_pump(true);

            println!("{:?}", &settings as *const _);
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

            let client = Client::new(ClientCallbacksImpl {
                life_span_handler: LifeSpanHandler::new(LifeSpanHandlerImpl {
                    proxy: Mutex::new(event_loop.create_proxy()),
                }),
                render_handler: RenderHandler::new(RenderHandlerCallbacksImpl {
                    pixel_buffer: Mutex::new(PixelBufferTyped::new_supported(width, height, &window)),
                    window,
                    popup_rect: Mutex::new(None),
                })
            });


            let browser = BrowserHost::create_browser_sync(
                &window_info,
                client,
                "https://www.google.com/",
                // "https://webkit.org/blog-files/3d-transforms/morphing-cubes.html",
                // "https://devyumao.github.io/dragon-loading/",
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
            let mut scheduled_work_queue = vec![];
            event_loop.run(move |event, _, control_flow| {
                match event {
                    Event::NewEvents(StartCause::ResumeTimeReached{..}) => {
                        while scheduled_work_queue.len() > 0 && scheduled_work_queue[0] <= Instant::now() {
                            scheduled_work_queue.remove(0);
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
                            WindowEvent::Touch(Touch{phase, location, force, id, ..}) => {
                                browser.get_host().send_touch_event(&TouchEvent {
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
                                })
                            }
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
                                println!("do scheduled work b {:?}", instant);
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
                    Event::EventsCleared => {
                        if scheduled_work_queue.len() > 0 {
                            *control_flow = ControlFlow::WaitUntil(scheduled_work_queue[0]);
                        } else {
                            *control_flow = ControlFlow::Wait;
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
