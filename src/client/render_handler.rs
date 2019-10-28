use crate::{
    accessibility_handler::AccessibilityHandler,
    browser::{Browser},
    browser_host::PaintElementType,
    drag::{DragData, DragOperation},
    refcounted::{RefCountedPtr, Wrapper},
    values::{Range, Rect, Point, Size},
    string::CefString,
};
use cef_sys::{
    cef_browser_t,
    cef_string_t,
    _cef_popup_features_t,
    cef_render_handler_t,
    cef_text_input_mode_t,
    cef_rect_t,
    cef_accessibility_handler_t,
    cef_screen_info_t,
    cef_range_t,
    cef_drag_operations_mask_t,
    cef_paint_element_type_t,
    cef_cursor_type_t,
    cef_cursor_info_t,
    cef_drag_data_t,
    HCURSOR,
};
use num_enum::UnsafeFromPrimitive;
use libc::c_int;
use std::os::raw::c_void;

#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, UnsafeFromPrimitive)]
pub enum TextInputMode {
    Default = cef_text_input_mode_t::CEF_TEXT_INPUT_MODE_DEFAULT,
    None = cef_text_input_mode_t::CEF_TEXT_INPUT_MODE_NONE,
    Text = cef_text_input_mode_t::CEF_TEXT_INPUT_MODE_TEXT,
    Tel = cef_text_input_mode_t::CEF_TEXT_INPUT_MODE_TEL,
    URL = cef_text_input_mode_t::CEF_TEXT_INPUT_MODE_URL,
    Email = cef_text_input_mode_t::CEF_TEXT_INPUT_MODE_EMAIL,
    Numeric = cef_text_input_mode_t::CEF_TEXT_INPUT_MODE_NUMERIC,
    Decimal = cef_text_input_mode_t::CEF_TEXT_INPUT_MODE_DECIMAL,
    Search = cef_text_input_mode_t::CEF_TEXT_INPUT_MODE_SEARCH,
}

#[derive(Copy, Clone, PartialEq)]
pub enum CursorType<'a> {
    Pointer,
    Cross,
    Hand,
    IBeam,
    Wait,
    Help,
    EastResize,
    NorthResize,
    NorthEastResize,
    NorthWestResize,
    SouthResize,
    SouthEastResize,
    SouthWestResize,
    WestResize,
    NorthSouthResize,
    EastWestResize,
    NorthEastSouthWestResize,
    NorthWestSouthEastResize,
    ColumnResize,
    RowResize,
    MiddlePanning,
    EastPanning,
    NorthPanning,
    NorthEastPanning,
    NorthWestPanning,
    SouthPanning,
    SouthEastPanning,
    SouthWestPanning,
    WestPanning,
    Move,
    VerticalText,
    Cell,
    ContextMenu,
    Alias,
    Progress,
    NoDrop,
    Copy,
    None,
    NotAllowed,
    ZoomIn,
    ZoomOut,
    Grab,
    Grabbing,
    Custom(CustomCursorInfo<'a>),
}

#[derive(Copy, Clone, PartialEq)]
pub struct CustomCursorInfo<'a> {
    pub hotspot: Point,
    pub image_scale_factor: f32,
    /// 8-bit BGRA cursor image
    pub buffer: &'a [u8],
    pub size: Size,
}

impl<'a> CursorType<'a> {
    unsafe fn from_raw(cursor_type: cef_cursor_type_t::Type, custom_cursor_info: *const cef_cursor_info_t) -> CursorType<'a> {
        match cursor_type {
            cef_cursor_type_t::CT_POINTER => Self::Pointer,
            cef_cursor_type_t::CT_CROSS => Self::Cross,
            cef_cursor_type_t::CT_HAND => Self::Hand,
            cef_cursor_type_t::CT_IBEAM => Self::IBeam,
            cef_cursor_type_t::CT_WAIT => Self::Wait,
            cef_cursor_type_t::CT_HELP => Self::Help,
            cef_cursor_type_t::CT_EASTRESIZE => Self::EastResize,
            cef_cursor_type_t::CT_NORTHRESIZE => Self::NorthResize,
            cef_cursor_type_t::CT_NORTHEASTRESIZE => Self::NorthEastResize,
            cef_cursor_type_t::CT_NORTHWESTRESIZE => Self::NorthWestResize,
            cef_cursor_type_t::CT_SOUTHRESIZE => Self::SouthResize,
            cef_cursor_type_t::CT_SOUTHEASTRESIZE => Self::SouthEastResize,
            cef_cursor_type_t::CT_SOUTHWESTRESIZE => Self::SouthWestResize,
            cef_cursor_type_t::CT_WESTRESIZE => Self::WestResize,
            cef_cursor_type_t::CT_NORTHSOUTHRESIZE => Self::NorthSouthResize,
            cef_cursor_type_t::CT_EASTWESTRESIZE => Self::EastWestResize,
            cef_cursor_type_t::CT_NORTHEASTSOUTHWESTRESIZE => Self::NorthEastSouthWestResize,
            cef_cursor_type_t::CT_NORTHWESTSOUTHEASTRESIZE => Self::NorthWestSouthEastResize,
            cef_cursor_type_t::CT_COLUMNRESIZE => Self::ColumnResize,
            cef_cursor_type_t::CT_ROWRESIZE => Self::RowResize,
            cef_cursor_type_t::CT_MIDDLEPANNING => Self::MiddlePanning,
            cef_cursor_type_t::CT_EASTPANNING => Self::EastPanning,
            cef_cursor_type_t::CT_NORTHPANNING => Self::NorthPanning,
            cef_cursor_type_t::CT_NORTHEASTPANNING => Self::NorthEastPanning,
            cef_cursor_type_t::CT_NORTHWESTPANNING => Self::NorthWestPanning,
            cef_cursor_type_t::CT_SOUTHPANNING => Self::SouthPanning,
            cef_cursor_type_t::CT_SOUTHEASTPANNING => Self::SouthEastPanning,
            cef_cursor_type_t::CT_SOUTHWESTPANNING => Self::SouthWestPanning,
            cef_cursor_type_t::CT_WESTPANNING => Self::WestPanning,
            cef_cursor_type_t::CT_MOVE => Self::Move,
            cef_cursor_type_t::CT_VERTICALTEXT => Self::VerticalText,
            cef_cursor_type_t::CT_CELL => Self::Cell,
            cef_cursor_type_t::CT_CONTEXTMENU => Self::ContextMenu,
            cef_cursor_type_t::CT_ALIAS => Self::Alias,
            cef_cursor_type_t::CT_PROGRESS => Self::Progress,
            cef_cursor_type_t::CT_NODROP => Self::NoDrop,
            cef_cursor_type_t::CT_COPY => Self::Copy,
            cef_cursor_type_t::CT_NONE => Self::None,
            cef_cursor_type_t::CT_NOTALLOWED => Self::NotAllowed,
            cef_cursor_type_t::CT_ZOOMIN => Self::ZoomIn,
            cef_cursor_type_t::CT_ZOOMOUT => Self::ZoomOut,
            cef_cursor_type_t::CT_GRAB => Self::Grab,
            cef_cursor_type_t::CT_GRABBING => Self::Grabbing,
            cef_cursor_type_t::CT_CUSTOM => Self::Custom({
                let cci = &*custom_cursor_info;
                CustomCursorInfo {
                    hotspot: Point::from(&cci.hotspot),
                    image_scale_factor: cci.image_scale_factor,
                    buffer: std::slice::from_raw_parts(cci.buffer as *const u8, (4 * cci.size.width * cci.size.height) as usize),
                    size: Size::from(&cci.size),
                }
            }),
            _ => panic!("bad custom cursor value"),
        }
    }
}

/// Screen information used when window rendering is disabled. This structure is
/// passed as a parameter to CefRenderHandler::GetScreenInfo and should be filled
/// in by the client.
pub struct ScreenInfo {
    /// Device scale factor. Specifies the ratio between physical and logical
    /// pixels.
    pub device_scale_factor: f32,

    /// The screen depth in bits per pixel.
    pub depth: u32,

    /// The bits per color component. This assumes that the colors are balanced
    /// equally.
    pub depth_per_component: u32,

    /// This can be `true` for black and white printers.
    pub is_monochrome: bool,

    /// This is set from the rcMonitor member of MONITORINFOEX, to whit:
    /// > A RECT structure that specifies the display monitor rectangle,
    /// > expressed in virtual-screen coordinates. Note that if the monitor
    /// > is not the primary display monitor, some of the rectangle's
    /// > coordinates may be negative values.
    ///
    /// The `rect` and `available_rect` properties are used to determine the
    /// available surface for rendering popup views.
    pub rect: Rect,

    /// This is set from the rcWork member of MONITORINFOEX, to whit:
    /// > A RECT structure that specifies the work area rectangle of the
    /// > display monitor that can be used by applications, expressed in
    /// > virtual-screen coordinates. Windows uses this rectangle to
    /// > maximize an application on the monitor. The rest of the area in
    /// > rcMonitor contains system windows such as the task bar and side
    /// > bars. Note that if the monitor is not the primary display monitor,
    /// > some of the rectangle's coordinates may be negative values.
    ///
    /// The `rect` and `available_rect` properties are used to determine the
    /// available surface for rendering popup views.
    pub available_rect: Rect,
}

impl ScreenInfo {
    fn write_to_cef(&self, cef: &mut cef_screen_info_t) {
        let ScreenInfo {
            device_scale_factor,
            depth,
            depth_per_component,
            is_monochrome,
            rect,
            available_rect,
        } = *self;
        cef.device_scale_factor = device_scale_factor;
        cef.depth = depth as _;
        cef.depth_per_component = depth_per_component as _;
        cef.is_monochrome = is_monochrome as _;
        cef.rect = rect.into();
        cef.available_rect = available_rect.into();
    }
}

impl From<&'_ cef_screen_info_t> for ScreenInfo {
    fn from(info: &cef_screen_info_t) -> ScreenInfo {
        ScreenInfo {
            device_scale_factor: info.device_scale_factor,
            depth: info.depth as u32,
            depth_per_component: info.depth as u32,
            is_monochrome: info.is_monochrome != 0,
            rect: Rect::from(&info.rect),
            available_rect: Rect::from(&info.available_rect),
        }
    }
}

ref_counted_ptr!{
    pub struct RenderHandler(*mut cef_render_handler_t);
}

impl RenderHandler {
    pub fn new<C: RenderHandlerCallbacks>(callbacks: C) -> RenderHandler {
        unsafe{ RenderHandler::from_ptr_unchecked(RenderHandlerWrapper(Box::new(callbacks)).wrap().into_raw()) }
    }
}

pub trait RenderHandlerCallbacks: 'static + Send + Sync {
    fn get_accessibility_handler(&self) -> Option<AccessibilityHandler> {
        None
    }
    fn get_root_screen_rect(
        &self,
        browser: Browser,
        rect: &mut Rect,
    ) -> bool {
        false
    }
    fn get_view_rect(
        &self,
        browser: Browser,
        rect: &mut Rect,
    ) {
    }
    fn get_screen_point(
        &self,
        browser: Browser,
        view_x: i32,
        view_y: i32,
        screen_x: &mut i32,
        screen_y: &mut i32,
    ) -> bool {
        false
    }
    fn get_screen_info(
        &self,
        browser: Browser,
        screen_info: &mut ScreenInfo,
    ) -> bool {
        false
    }
    fn on_popup_show(
        &self,
        browser: Browser,
        show: i32,
    ) {
    }
    fn on_popup_size(
        &self,
        browser: Browser,
        rect: Rect,
    ) {
    }
    fn on_paint(
        &self,
        browser: Browser,
        type_: PaintElementType,
        dirty_rects_count: usize,
        dirty_rects: Rect,
        buffer: &[u8],
        width: i32,
        height: i32,
    ) {
    }
    fn on_accelerated_paint(
        &self,
        browser: Browser,
        type_: PaintElementType,
        dirty_rects: &[Rect],
        shared_handle: *mut c_void,
    ) {
    }
    fn on_cursor_change(
        &self,
        browser: Browser,
        cursor: HCURSOR, // TODO: GENERALIZE TO CROSS-PLATFORM CURSOR TYPE
        type_: CursorType<'_>,
    ) {
    }
    fn start_dragging(
        &self,
        browser: Browser,
        drag_data: DragData,
        allowed_ops: DragOperation,
        x: i32,
        y: i32,
    ) -> bool {
        false
    }
    fn update_drag_cursor(
        &self,
        browser: Browser,
        operation: DragOperation,
    ) {
    }
    fn on_scroll_offset_changed(
        &self,
        browser: Browser,
        x: f64,
        y: f64,
    ) {
    }
    fn on_ime_composition_range_changed(
        &self,
        browser: Browser,
        selected_range: Range,
        character_bounds_count: usize,
        character_bounds: Rect,
    ) {
    }
    fn on_text_selection_changed(
        &self,
        browser: Browser,
        selected_text: &str,
        selected_range: Range,
    ) {
    }
    fn on_virtual_keyboard_requested(
        &self,
        browser: Browser,
        input_mode: TextInputMode,
    ) {
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PopupFeatures {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub menu_bar_visible: bool,
    pub status_bar_visible: bool,
    pub tool_bar_visible: bool,
    pub scroll_bars_visible: bool,
}

impl PopupFeatures {
    pub fn new(popup_features: &_cef_popup_features_t) -> PopupFeatures {
        PopupFeatures {
            x: match popup_features.xSet {
                0 => None,
                _ => Some(popup_features.x),
            },
            y: match popup_features.ySet {
                0 => None,
                _ => Some(popup_features.y),
            },
            width: match popup_features.widthSet {
                0 => None,
                _ => Some(popup_features.width),
            },
            height: match popup_features.heightSet {
                0 => None,
                _ => Some(popup_features.height),
            },
            menu_bar_visible: popup_features.menuBarVisible != 0,
            status_bar_visible: popup_features.statusBarVisible != 0,
            tool_bar_visible: popup_features.toolBarVisible != 0,
            scroll_bars_visible: popup_features.scrollbarsVisible != 0,
        }
    }
}

pub struct RenderHandlerWrapper(Box<dyn RenderHandlerCallbacks>);

impl Wrapper for RenderHandlerWrapper {
    type Cef = cef_render_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_render_handler_t {
                base: unsafe { std::mem::zeroed() },
                get_accessibility_handler: Some(RenderHandlerWrapper::get_accessibility_handler),
                get_root_screen_rect: Some(RenderHandlerWrapper::get_root_screen_rect),
                get_view_rect: Some(RenderHandlerWrapper::get_view_rect),
                get_screen_point: Some(RenderHandlerWrapper::get_screen_point),
                get_screen_info: Some(RenderHandlerWrapper::get_screen_info),
                on_popup_show: Some(RenderHandlerWrapper::on_popup_show),
                on_popup_size: Some(RenderHandlerWrapper::on_popup_size),
                on_paint: Some(RenderHandlerWrapper::on_paint),
                on_accelerated_paint: Some(RenderHandlerWrapper::on_accelerated_paint),
                on_cursor_change: Some(RenderHandlerWrapper::on_cursor_change),
                start_dragging: Some(RenderHandlerWrapper::start_dragging),
                update_drag_cursor: Some(RenderHandlerWrapper::update_drag_cursor),
                on_scroll_offset_changed: Some(RenderHandlerWrapper::on_scroll_offset_changed),
                on_ime_composition_range_changed: Some(RenderHandlerWrapper::on_ime_composition_range_changed),
                on_text_selection_changed: Some(RenderHandlerWrapper::on_text_selection_changed),
                on_virtual_keyboard_requested: Some(RenderHandlerWrapper::on_virtual_keyboard_requested),
                ..unsafe { std::mem::zeroed() }
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for RenderHandlerWrapper: cef_render_handler_t {
        fn get_accessibility_handler(&self) -> *mut cef_accessibility_handler_t {
            unimplemented!()
        }
        fn get_root_screen_rect(
            &self,
            browser: Browser: *mut cef_browser_t,
            rect: &mut Rect: *mut cef_rect_t,
        ) -> c_int {
            unimplemented!()
        }
        fn get_view_rect(
            &self,
            browser: Browser: *mut cef_browser_t,
            rect: &mut Rect: *mut cef_rect_t,
        ) {
            unimplemented!()
        }
        fn get_screen_point(
            &self,
            browser: Browser: *mut cef_browser_t,
            view_x: i32: c_int,
            view_y: i32: c_int,
            screen_x: &mut i32: *mut c_int,
            screen_y: &mut i32: *mut c_int,
        ) -> c_int {
            unimplemented!()
        }
        fn get_screen_info(
            &self,
            browser: Browser: *mut cef_browser_t,
            screen_info: *mut cef_screen_info_t: *mut cef_screen_info_t,
        ) -> c_int {
            unimplemented!()
        }
        fn on_popup_show(
            &self,
            browser: Browser: *mut cef_browser_t,
            show: i32: c_int,
        ) {
            unimplemented!()
        }
        fn on_popup_size(
            &self,
            browser: Browser: *mut cef_browser_t,
            rect: &Rect: *const cef_rect_t,
        ) {
            unimplemented!()
        }
        fn on_paint(
            &self,
            browser: Browser: *mut cef_browser_t,
            type_: PaintElementType: cef_paint_element_type_t::Type,
            dirty_rects_count: usize: usize,
            dirty_rects: &Rect: *const cef_rect_t,
            buffer: *const c_void: *const c_void,
            width: i32: c_int,
            height: i32: c_int,
        ) {
            unimplemented!()
        }
        fn on_accelerated_paint(
            &self,
            browser: Browser: *mut cef_browser_t,
            type_: PaintElementType: cef_paint_element_type_t::Type,
            dirty_rects_count: usize: usize,
            dirty_rects: *const cef_rect_t: *const cef_rect_t,
            shared_handle: *mut c_void: *mut c_void,
        ) {
            unimplemented!()
        }
        fn on_cursor_change(
            &self,
            browser: Browser: *mut cef_browser_t,
            cursor: HCURSOR: HCURSOR, // TODO: GENERALIZE TO CROSS-PLATFORM CURSOR TYPE
            type_: cef_cursor_type_t::Type: cef_cursor_type_t::Type,
            custom_cursor_info: *const cef_cursor_info_t: *const cef_cursor_info_t,
        ) {
            unimplemented!()
        }
        fn start_dragging(
            &self,
            browser: Browser: *mut cef_browser_t,
            drag_data: DragData: *mut cef_drag_data_t,
            allowed_ops: DragOperation: cef_drag_operations_mask_t,
            x: i32: c_int,
            y: i32: c_int,
        ) -> c_int {
            unimplemented!()
        }
        fn update_drag_cursor(
            &self,
            browser: Browser: *mut cef_browser_t,
            operation: DragOperation: cef_drag_operations_mask_t,
        ) {
            unimplemented!()
        }
        fn on_scroll_offset_changed(
            &self,
            browser: Browser: *mut cef_browser_t,
            x: f64: f64,
            y: f64: f64,
        ) {
            unimplemented!()
        }
        fn on_ime_composition_range_changed(
            &self,
            browser: Browser: *mut cef_browser_t,
            selected_range: &Range: *const cef_range_t,
            character_bounds_count: usize: usize,
            character_bounds: &Rect: *const cef_rect_t,
        ) {
            unimplemented!()
        }
        fn on_text_selection_changed(
            &self,
            browser: Browser: *mut cef_browser_t,
            selected_text: &CefString: *const cef_string_t,
            selected_range: &Range: *const cef_range_t,
        ) {
            unimplemented!()
        }
        fn on_virtual_keyboard_requested(
            &self,
            browser: Browser: *mut cef_browser_t,
            input_mode: TextInputMode: cef_text_input_mode_t::Type,
        ) {
            unimplemented!()
        }
    }
}
