use cef_sys::{cef_browser_host_create_browser, cef_browser_host_t, cef_paint_element_type_t};
use num_enum::UnsafeFromPrimitive;
use std::{collections::HashMap, ptr::null_mut};
use winapi::shared::minwindef::HINSTANCE;

use crate::{
    browser::{Browser, BrowserSettings},
    client::{Client, ClientWrapper},
    drag::{DragData, DragOperation},
    events::{KeyEvent, MouseButtonType, MouseEvent, TouchEvent},
    file_dialog::{FileDialogMode},
    image::Image,
    ime::CompositionUnderline,
    navigation::NavigationEntry,
    printing::PDFPrintSettings,
    request_context::RequestContext,
    string::CefString,
    values::{DictionaryValue, Point, Range, StoredValue},
    window::WindowInfo,
    extension::Extension,
};

/// Paint element types.
#[repr(i32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, UnsafeFromPrimitive)]
pub enum PaintElementType {
    View = cef_paint_element_type_t::PET_VIEW as i32,
    Popup = cef_paint_element_type_t::PET_POPUP as i32,
}

#[cfg(target_os = "windows")]
pub type WindowHandle = HINSTANCE;
#[cfg(target_os = "linux")]
pub type WindowHandle = u64;
#[cfg(target_os = "macos")]
pub type WindowHandle = *mut std::ffi::c_void; // Actually NSView*

ref_counted_ptr! {
    /// Structure used to represent the browser process aspects of a browser window.
    /// The functions of this structure can only be called in the browser process.
    /// They may be called on any thread in that process unless otherwise indicated
    /// in the comments.
    pub struct BrowserHost(*mut cef_browser_host_t);
}

impl BrowserHost {
    /// Create a new browser window using the window parameters specified by
    /// `window_info`. All values will be copied internally and the actual window will
    /// be created on the UI thread. If `request_context` is None the global request
    /// context will be used. This function can be called on any browser process
    /// thread and will not block. The optional `extra_info` parameter provides an
    /// opportunity to specify extra information specific to the created browser that
    /// will be passed to [RenderProcessHandler::on_browser_created] in the
    /// render process.
    pub fn create_browser<C: Client + 'static>(
        window_info: &WindowInfo,
        client: C,
        url: &str,
        settings: &BrowserSettings,
        extra_info: Option<&HashMap<String, StoredValue>>,
        request_context: Option<&RequestContext>,
    ) -> bool {
        let extra_info = extra_info.and_then(|ei| Some(DictionaryValue::from(ei)));
        let client = ClientWrapper::wrap(client);

        unsafe {
            cef_browser_host_create_browser(
                window_info.get(),
                client,
                CefString::new(url).as_ref(),
                settings.get(),
                extra_info
                    .and_then(|ei| Some(ei.as_ptr()))
                    .unwrap_or_else(null_mut),
                request_context
                    .and_then(|rc| Some(rc.as_ptr()))
                    .unwrap_or_else(null_mut),
            ) != 0
        }
    }
    /// Returns the hosted browser object.
    pub fn get_browser(&self) -> Browser {
        unimplemented!()
    }
    /// Request that the browser close. The JavaScript 'onbeforeunload' event will
    /// be fired. If `force_close` is false the event handler, if any, will be
    /// allowed to prompt the user and the user can optionally cancel the close. If
    /// `force_close` is true the prompt will not be displayed and the close
    /// will proceed. Results in a call to [LifeSpanHandler::do_close] if
    /// the event handler allows the close or if `force_close` is true. See
    /// [LifeSpanHandler::do_close] documentation for additional usage
    /// information.
    pub fn close_browser(&self, force_close: bool) {
        unimplemented!()
    }
    /// Helper for closing a browser. Call this function from the top-level window
    /// close handler. Internally this calls CloseBrowser(false) if the close
    /// has not yet been initiated. This function returns false while the close
    /// is pending and true after the close has completed. See [close_browser]
    /// and [LifeSpanHandler::do_close] documentation for additional usage
    /// information. This function must be called on the browser process UI thread.
    pub fn try_close_browser(&self) -> bool {
        unimplemented!()
    }
    /// Set whether the browser is focused.
    pub fn set_focus(&mut self, focus: bool) {
        unimplemented!()
    }
    /// Retrieve the window handle for this browser. If this browser is wrapped in
    /// a [BrowserView] this function should be called on the browser process
    /// UI thread and it will return the handle for the top-level native window.
    pub fn get_window_handle(&self) -> WindowHandle {
        unimplemented!()
    }
    /// Retrieve the window handle of the browser that opened this browser. Will
    /// return None for non-popup windows or if this browser is wrapped in a
    /// [BrowserView]. This function can be used in combination with custom
    /// handling of modal windows.
    pub fn get_opener_window_handle(&self) -> Option<WindowHandle> {
        unimplemented!()
    }
    /// Returns true if this browser is wrapped in a [BrowserView].
    pub fn has_view(&self) -> bool {
        unimplemented!()
    }
    /// Returns the client for this browser.
    pub fn get_client(&self) -> Option<Box<dyn Client>> {
        unimplemented!()
    }
    /// Returns the request context for this browser.
    pub fn get_request_context(&self) -> RequestContext {
        unimplemented!()
    }
    /// Get the current zoom level. The default zoom level is 0.0. This function
    /// can only be called on the UI thread.
    pub fn get_zoom_level(&self) -> f64 {
        unimplemented!()
    }
    /// Change the zoom level to the specified value. Specify 0.0 to reset the zoom
    /// level. If called on the UI thread the change will be applied immediately.
    /// Otherwise, the change will be applied asynchronously on the UI thread.
    pub fn set_zoom_level(&mut self) {
        unimplemented!()
    }
    /// Call to run a file chooser dialog. Only a single file chooser dialog may be
    /// pending at any given time. `mode` represents the type of dialog to display.
    /// `title` to the title to be used for the dialog and may be None to show the
    /// default title ("Open" or "Save" depending on the mode). `default_file_path`
    /// is the path with optional directory and/or file name component that will be
    /// initially selected in the dialog. `accept_filters` are used to restrict the
    /// selectable file types and may any combination of (a) valid lower-cased MIME
    /// types (e.g. "text/*" or "image/\*"), (b) individual file extensions (e.g.
    /// ".txt" or ".png"), or (c) combined description and file extension delimited
    /// using "|" and ";" (e.g. "Image Types|.png;.gif;.jpg").
    /// `selected_accept_filter` is the 0-based index of the filter that will be
    /// selected by default. `callback` will be executed after the dialog is
    /// dismissed or immediately if another dialog is already pending. The dialog
    /// will be initiated asynchronously on the UI thread.
    ///
    /// On the `callback`, the first parameter is the 0-based index of the value
    /// selected from `accept_filters`. The second parameter will be a single value
    /// or a list of values depending on the dialog mode. If the selection was
    /// cancelled it will be None.
    pub fn run_file_dialog<F: FnOnce(usize, Option<Vec<String>>)>(
        &self,
        mode: FileDialogMode,
        title: Option<&str>,
        default_file_path: Option<&str>,
        accept_filters: &[&str],
        selected_accept_filter: i32,
        callback: F,
    ) {
        // RunFileDialogCallbackWrapper!
        unimplemented!()
    }
    /// Download the file at `url` using [DownloadHandler].
    pub fn start_download(&mut self, url: &str) {
        unimplemented!()
    }
    /// Download `image_url` and execute `callback` on completion with the images
    /// received from the renderer. If `is_favicon` is true then cookies are
    /// not sent and not accepted during download. Images with density independent
    /// pixel (DIP) sizes larger than `max_image_size` are filtered out from the
    /// image results. Versions of the image at different scale factors may be
    /// downloaded up to the maximum scale factor supported by the system. If there
    /// are no image results <= `max_image_size` then the smallest image is resized
    /// to `max_image_size` and is the only result. A `max_image_size` of 0 means
    /// unlimited. If `bypass_cache` is true then `image_url` is requested from
    /// the server even if it is present in the browser cache.
    ///
    /// On the callback, the first parameter is the URL that was downloaded, the
    /// second parameter is the resulting HTTP status code and the third is the
    /// resulting image, possibly None if the download failed.
    pub fn download_image<F: FnOnce(&str, u16, Option<Image>)>(
        &self,
        image_url: &str,
        is_favicon: bool,
        max_image_size: u32,
        bypass_cache: bool,
        callback: F,
    ) {
        unimplemented!()
    }
    /// Print the current browser contents.
    pub fn print(&self) {
        unimplemented!()
    }
    /// Print the current browser contents to the PDF file specified by `path` and
    /// execute `callback` on completion. The caller is responsible for deleting
    /// `path` when done. For PDF printing to work on Linux you must implement the
    /// [PrintHandler::GetPdfPaperSize] function.
    ///
    /// On the callback, the first parameter is the output path. The second parameter
    /// will be true if the printing completed successfully or false otherwise.
    pub fn print_to_pdf<F: FnOnce(&str, bool)>(
        &self,
        path: &str,
        settings: PDFPrintSettings,
        callback: F,
    ) {
        unimplemented!()
    }
    /// Search for `searchText`. `identifier` must be a unique ID and these IDs
    /// must strictly increase so that newer requests always have greater IDs than
    /// older requests. If `identifier` is zero or less than the previous ID value
    /// then it will be automatically assigned a new valid ID. `forward` indicates
    /// whether to search forward or backward within the page. `match_case`
    /// indicates whether the search should be case-sensitive. `find_next` indicates
    /// whether this is the first request or a follow-up. The [FindHandler]
    /// instance, if any, returned via [Client::get_find_handler] will be called
    /// to report find results.
    pub fn find(
        &self,
        identifier: i32,
        search_text: &str,
        forward: bool,
        match_case: bool,
        find_next: bool,
    ) {
        unimplemented!()
    }
    /// Cancel all searches that are currently going on.
    pub fn stop_finding(&self, clear_selection: bool) {
        unimplemented!()
    }
    /// Open developer tools (DevTools) in its own browser. The DevTools browser
    /// will remain associated with this browser. If the DevTools browser is
    /// already open then it will be focused, in which case the `window_info`,
    /// `client` and `settings` parameters will be ignored. If `inspect_element_at`
    /// is non-None then the element at the specified (x,y) location will be
    /// inspected. The `window_info` parameter will be ignored if this browser is
    /// wrapped in a [BrowserView].
    pub fn show_dev_tools(
        &self,
        window_info: &WindowInfo,
        client: Option<Box<dyn Client>>,
        settings: Option<BrowserSettings>,
        inspect_element_at: Point,
    ) {
        unimplemented!()
    }
    /// Explicitly close the associated DevTools browser, if any.
    pub fn close_dev_tools(&self) {
        unimplemented!()
    }
    /// Returns true if this browser currently has an associated DevTools
    /// browser. Must be called on the browser process UI thread.
    pub fn has_dev_tools(&self) -> bool {
        unimplemented!()
    }
    /// Retrieve a snapshot of current navigation entries as values sent to the
    /// specified visitor. If `current_only` is true only the current
    /// navigation entry will be sent, otherwise all navigation entries will be
    /// sent.
    ///
    /// The visitor will be called on the browser process UI thread.
    /// The first parameter is the navigation entry at the position given in the
    /// third parameter. The second parameter indicates whether it's the currently
    /// loaded navigation entry and the fourth parameter is the total number of
    /// entries. Return true to continue visiting entries or false to stop.
    pub fn get_navigation_entries<F: Fn(&NavigationEntry, bool, usize, usize) -> bool>(
        &self,
        visitor: F,
        current_only: bool,
    ) {
        unimplemented!()
    }
    /// Set whether mouse cursor change is disabled.
    pub fn set_mouse_cursor_change_disabled(&mut self, disabled: bool) {
        unimplemented!()
    }
    /// Returns true if mouse cursor change is disabled.
    pub fn is_mouse_cursor_change_disabled(&self) -> bool {
        unimplemented!()
    }
    /// If a misspelled word is currently selected in an editable node calling this
    /// function will replace it with the specified `word`.
    pub fn replace_misspelling(&mut self, word: &str) {
        unimplemented!()
    }
    /// Add the specified `word` to the spelling dictionary.
    pub fn add_word_to_dictionary(&mut self, word: &str) {
        unimplemented!()
    }
    /// Returns true if window rendering is disabled.
    pub fn is_window_rendering_disabled(&self) -> bool {
        unimplemented!()
    }
    /// Notify the browser that the widget has been resized. The browser will first
    /// call [RenderHandler::get_view_rect] to get the new size and then call
    /// [RenderHandler::on_paint] asynchronously with the updated regions. This
    /// function is only used when window rendering is disabled.
    pub fn was_resized(&self) {
        unimplemented!()
    }
    /// Notify the browser that it has been hidden or shown. Layouting and
    /// [RenderHandler::on_paint] notification will stop when the browser is
    /// hidden. This function is only used when window rendering is disabled.
    pub fn was_hidden(&self, hidden: bool) {
        unimplemented!()
    }
    /// Send a notification to the browser that the screen info has changed. The
    /// browser will then call [RenderHandler::get_screen_info] to update the
    /// screen information with the new values. This simulates moving the webview
    /// window from one display to another, or changing the properties of the
    /// current display. This function is only used when window rendering is
    /// disabled.
    pub fn notify_screen_info_changed(&self) {
        unimplemented!()
    }
    /// Invalidate the view. The browser will call [RenderHandler::on_paint]
    /// asynchronously. This function is only used when window rendering is
    /// disabled.
    pub fn invalidate(&mut self, element_type: PaintElementType) {
        unimplemented!()
    }
    /// Issue a BeginFrame request to Chromium.  Only valid when
    /// [WindowInfo::external_begin_frame_enabled] is set to true.
    pub fn send_external_begin_frame(&mut self) {
        unimplemented!()
    }
    /// Send a key event to the browser.
    pub fn send_key_event(&mut self, event: &KeyEvent) {
        unimplemented!()
    }
    /// Send a mouse click event to the browser. The `x` and `y` coordinates are
    /// relative to the upper-left corner of the view.
    pub fn send_mouse_click_event(
        &mut self,
        event: &MouseEvent,
        button_type: MouseButtonType,
        mouse_up: bool,
        click_count: i32,
    ) {
        unimplemented!()
    }
    /// Send a mouse move event to the browser. The `x` and `y` coordinates are
    /// relative to the upper-left corner of the view.
    pub fn send_mouse_move_event(&mut self, event: &MouseEvent, mouse_leave: bool) {
        unimplemented!()
    }
    /// Send a mouse wheel event to the browser. The `x` and `y` coordinates are
    /// relative to the upper-left corner of the view. The `deltaX` and `deltaY`
    /// values represent the movement delta in the X and Y directions respectively.
    /// In order to scroll inside select popups with window rendering disabled
    /// [RenderHandler::get_screen_point] should be implemented properly.
    pub fn send_mouse_wheel_event(&mut self, event: &MouseEvent, delta_x: i32, delta_y: i32) {
        unimplemented!()
    }
    /// Send a touch event to the browser for a windowless browser.
    pub fn send_touch_event(&mut self, event: &TouchEvent) {
        unimplemented!()
    }
    /// Send a focus event to the browser.
    pub fn send_focus_event(&mut self, set_focus: bool) {
        unimplemented!()
    }
    /// Send a capture lost event to the browser.
    pub fn send_capture_lost_event(&mut self) {
        unimplemented!()
    }
    /// Notify the browser that the window hosting it is about to be moved or
    /// resized. This function is only used on Windows and Linux.
    pub fn notify_move_or_resize_started(&self) {
        unimplemented!()
    }
    /// Returns the maximum rate in frames per second (fps) that
    /// [RenderHandler::on_paint] will be called for a windowless browser. The
    /// actual fps may be lower if the browser cannot generate frames at the
    /// requested rate. The minimum value is 1 and the maximum value is 60 (default
    /// 30). This function can only be called on the UI thread.
    pub fn get_windowless_frame_rate(&self) -> i32 {
        unimplemented!()
    }
    // Set the maximum rate in frames per second (fps) that [RenderHandler::on_paint]
    // will be called for a windowless browser. The actual fps may be
    // lower if the browser cannot generate frames at the requested rate. The
    // minimum value is 1 and the maximum value is 60 (default 30). Can also be
    // set at browser creation via [BrowserSettings::windowless_frame_rate].
    pub fn set_windowless_frame_rate(&mut self, frame_rate: i32) {
        unimplemented!()
    }
    /// Begins a new composition or updates the existing composition. Blink has a
    /// special node (a composition node) that allows the input function to change
    /// text without affecting other DOM nodes. `text` is the optional text that
    /// will be inserted into the composition node. `underlines` is an optional set
    /// of ranges that will be underlined in the resulting text.
    /// `replacement_range` is an optional range of the existing text that will be
    /// replaced. `selection_range` is an optional range of the resulting text that
    /// will be selected after insertion or replacement. The `replacement_range`
    /// value is only used on OS X.
    ///
    /// This function may be called multiple times as the composition changes. When
    /// the client is done making changes the composition should either be canceled
    /// or completed. To cancel the composition call [BrowserHost::ime_cancel_composition]. To
    /// complete the composition call either [BrowserHost::ime_commit_text] or
    /// [BrowserHost::ime_finish_composing_text]. Completion is usually signaled when:
    ///   A. The client receives a WM_IME_COMPOSITION message with a GCS_RESULTSTR
    ///      flag (on Windows), or;
    ///   B. The client receives a "commit" signal of GtkIMContext (on Linux), or;
    ///   C. insertText of NSTextInput is called (on Mac).
    ///
    /// This function is only used when window rendering is disabled.
    pub fn ime_set_composition(
        &mut self,
        text: &str,
        underlines_count: usize,
        underlines: &CompositionUnderline,
        replacement_range: Range,
        selection_range: Range,
    ) {
        unimplemented!()
    }

    /// Completes the existing composition by optionally inserting the specified
    /// `text` into the composition node. `replacement_range` is an optional range
    /// of the existing text that will be replaced. `relative_cursor_pos` is where
    /// the cursor will be positioned relative to the current cursor position. See
    /// comments on [BrowserHost::ime_set_composition] for usage. The `replacement_range` and
    /// `relative_cursor_pos` values are only used on OS X. This function is only
    /// used when window rendering is disabled.
    pub fn ime_commit_text(
        &mut self,
        text: Option<&str>,
        replacement_range: Option<Range>,
        relative_cursor_pos: i32,
    ) {
        unimplemented!()
    }
    /// Completes the existing composition by applying the current composition node
    /// contents. If `keep_selection` is false the current selection, if any,
    /// will be discarded. See comments on [BrowserHost::ime_set_composition] for usage. This
    /// function is only used when window rendering is disabled.
    pub fn ime_finish_composing_text(&mut self, keep_selection: bool) {
        unimplemented!()
    }
    /// Cancels the existing composition and discards the composition node contents
    /// without applying them. See comments on ImeSetComposition for usage. This
    /// function is only used when window rendering is disabled.
    pub fn ime_cancel_composition(&mut self) {
        unimplemented!()
    }
    /// Call this function when the user drags the mouse into the web view (before
    /// calling [BrowserHost::drag_target_drag_over]/[BrowserHost::drag_target_leave]/[BrowserHost::drag_target_drop]). `drag_data`
    /// should not contain file contents as this type of data is not allowed to be
    /// dragged into the web view. File contents can be removed using
    /// [DragData::reset_file_contents] (for example, if `drag_data` comes from
    /// [RenderHandler::start_dragging]). This function is only used when
    /// window rendering is disabled.
    pub fn drag_target_drag_enter(
        &mut self,
        drag_data: &DragData,
        event: &MouseEvent,
        allowed_ops: &[DragOperation],
    ) {
        unimplemented!()
    }
    /// Call this function each time the mouse is moved across the web view during
    /// a drag operation (after calling [BrowserHost::drag_target_drag_enter] and before calling
    /// [BrowserHost::drag_target_drag_leave]/[BrowserHost::drag_target_drop]). This function is only used when window
    /// rendering is disabled.
    pub fn drag_target_drag_over(&mut self, event: &MouseEvent, allowed_ops: &[DragOperation]) {
        unimplemented!()
    }
    /// Call this function when the user drags the mouse out of the web view (after
    /// calling [BrowserHost::drag_target_drag_enter]). This function is only used when window
    /// rendering is disabled.
    pub fn drag_target_drag_leave(&mut self) {
        unimplemented!()
    }
    /// Call this function when the user completes the drag operation by dropping
    /// the object onto the web view (after calling [BrowserHost::drag_target_drag_enter]). The
    /// object being dropped is `drag_data`, given as an argument to the previous
    /// [BrowserHost::drag_target_drag_enter] call. This function is only used when window rendering
    /// is disabled.
    pub fn drag_target_drop(&mut self, event: &MouseEvent) {
        unimplemented!()
    }
    /// Call this function when the drag operation started by a
    /// [RenderHandler::start_dragging] call has ended either in a drop or by
    /// being cancelled. `x` and `y` are mouse coordinates relative to the upper-
    /// left corner of the view. If the web view is both the drag source and the
    /// drag target then all drag_target_* functions should be called before
    /// drag_source_* methods. This function is only used when window rendering is
    /// disabled.
    pub fn drag_source_ended_at(&mut self, x: i32, y: i32, op: &[DragOperation]) {
        unimplemented!()
    }
    /// Call this function when the drag operation started by a
    /// [RenderHandler::start_dragging] call has completed. This function may
    /// be called immediately without first calling [BrowserHost::drag_source_ended_at] to cancel a
    /// drag operation. If the web view is both the drag source and the drag target
    /// then all drag_target_* functions should be called before drag_source_* methods.
    /// This function is only used when window rendering is disabled.
    pub fn drag_source_system_drag_ended(&mut self) {
        unimplemented!()
    }
    /// Returns the current visible navigation entry for this browser. This
    /// function can only be called on the UI thread.
    pub fn get_visible_navigation_entry(&self) -> NavigationEntry {
        unimplemented!()
    }
    // TODO: continue
}
