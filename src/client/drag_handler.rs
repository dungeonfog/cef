use crate::{
    browser::{Browser},
    drag::{DragData, DragOperation},
    frame::Frame,
    refcounted::{RefCountedPtr, Wrapper},
    values::{Rect},
};
use cef_sys::{
    cef_browser_t,
    cef_drag_data_t,
    cef_drag_operations_mask_t,
    cef_frame_t,
    _cef_popup_features_t,
    cef_drag_handler_t,
    cef_draggable_region_t,
    cef_rect_t,
};
use std::os::raw::c_int;

ref_counted_ptr!{
    pub struct DragHandler(*mut cef_drag_handler_t);
}

impl DragHandler {
    pub fn new<C: DragHandlerCallbacks>(callbacks: C) -> DragHandler {
        unsafe{ DragHandler::from_ptr_unchecked(DragHandlerWrapper(Box::new(callbacks)).wrap().into_raw()) }
    }
}

pub trait DragHandlerCallbacks: 'static + Send + Sync {
    fn on_drag_enter(
        &self,
        _browser: Browser,
        _drag_data: DragData,
        _mask: DragOperation,
    ) -> bool {
        false
    }
    fn on_draggable_regions_changed(
        &self,
        browser: Browser,
        frame: Frame,
        regions: &[DraggableRegion],
    ) {
    }
}

#[repr(transparent)]
pub struct DraggableRegion(cef_draggable_region_t);

impl DraggableRegion {
    pub fn bounds(&self) -> Rect {
        unsafe{ std::mem::transmute(cef_rect_t{ ..self.0.bounds }) }
    }

    pub fn draggable(&self) -> bool {
        self.0.draggable != 0
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

struct DragHandlerWrapper(Box<dyn DragHandlerCallbacks>);

impl Wrapper for DragHandlerWrapper {
    type Cef = cef_drag_handler_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_drag_handler_t {
                base: unsafe { std::mem::zeroed() },
                on_drag_enter: Some(Self::on_drag_enter),
                on_draggable_regions_changed: Some(Self::on_draggable_regions_changed),
            },
            self,
        )
    }
}

cef_callback_impl!{
    impl for DragHandlerWrapper: cef_drag_handler_t {
        fn on_drag_enter(
            &self,
            browser: Browser: *mut cef_browser_t,
            drag_data: DragData: *mut cef_drag_data_t,
            mask: DragOperation: cef_drag_operations_mask_t
        ) -> c_int {
            self.0.on_drag_enter(browser, drag_data, mask) as _
        }
        fn on_draggable_regions_changed(
            &self,
            browser: Browser: *mut cef_browser_t,
            frame: Frame: *mut cef_frame_t,
            regions_count: usize: usize,
            regions: *const cef_draggable_region_t: *const cef_draggable_region_t
        ) {
            let regions = unsafe{ std::slice::from_raw_parts(regions as *const DraggableRegion, regions_count) };
            self.0.on_draggable_regions_changed(browser, frame, regions)
        }
    }
}
