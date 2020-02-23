use cef_sys::{cef_drag_data_create, cef_drag_data_t, cef_drag_operations_mask_t};
use std::ptr;
use crate::{
    string::CefString,
    image::Image,
    stream::StreamWriter,
    string::CefStringList,
    values::Point,
};
use bitflags::bitflags;

bitflags!{
    /// "Verb" of a drag-and-drop operation as negotiated between the source and
    /// destination. These constants match their equivalents in WebCore's
    /// DragActions.h.
    pub struct DragOperation: crate::CEnumType {
        const NONE = cef_drag_operations_mask_t::DRAG_OPERATION_NONE.0;
        const COPY = cef_drag_operations_mask_t::DRAG_OPERATION_COPY.0;
        const LINK = cef_drag_operations_mask_t::DRAG_OPERATION_LINK.0;
        const GENERIC = cef_drag_operations_mask_t::DRAG_OPERATION_GENERIC.0;
        const PRIVATE = cef_drag_operations_mask_t::DRAG_OPERATION_PRIVATE.0;
        const MOVE = cef_drag_operations_mask_t::DRAG_OPERATION_MOVE.0;
        const DELETE = cef_drag_operations_mask_t::DRAG_OPERATION_DELETE.0;
    }
}

impl DragOperation {
    pub unsafe fn from_unchecked(c: crate::CEnumType) -> Self {
        std::mem::transmute(c)
    }
}

ref_counted_ptr! {
    /// Structure used to represent drag data. The functions of this structure may be
    /// called on any thread.
    pub struct DragData(*mut cef_drag_data_t);
}

impl DragData {
    pub fn new() -> Self {
        unsafe { Self::from_ptr_unchecked(cef_drag_data_create()) }
    }

    /// Returns `true` if this object is read-only.
    pub fn is_read_only(&self) -> bool {
        unsafe { self.0.is_read_only.unwrap()(self.as_ptr()) != 0 }
    }
    /// Returns `true` if the drag data is a link.
    pub fn is_link(&self) -> bool {
        unsafe { self.0.is_link.unwrap()(self.as_ptr()) != 0 }
    }
    /// Returns `true` if the drag data is a text or html fragment.
    pub fn is_fragment(&self) -> bool {
        unsafe { self.0.is_fragment.unwrap()(self.as_ptr()) != 0 }
    }
    /// Returns `true` if the drag data is a file.
    pub fn is_file(&self) -> bool {
        unsafe { self.0.is_file.unwrap()(self.as_ptr()) != 0 }
    }
    /// Return the link URL that is being dragged.
    pub fn get_link_url(&self) -> Option<String> {
        unsafe { CefString::from_userfree(self.0.get_link_url.unwrap()(self.as_ptr())).map(String::from) }
    }
    /// Return the title associated with the link being dragged.
    pub fn get_link_title(&self) -> Option<String> {
        unsafe { CefString::from_userfree(self.0.get_link_title.unwrap()(self.as_ptr())).map(String::from) }
    }
    /// Return the metadata, if any, associated with the link being dragged.
    pub fn get_link_metadata(&self) -> Option<String> {
        unsafe { CefString::from_userfree(self.0.get_link_metadata.unwrap()(self.as_ptr())).map(String::from) }
    }
    /// Return the plain text fragment that is being dragged.
    pub fn get_fragment_text(&self) -> Option<String> {
        unsafe { CefString::from_userfree(self.0.get_fragment_text.unwrap()(self.as_ptr())).map(String::from) }
    }
    /// Return the text/html fragment that is being dragged.
    pub fn get_fragment_html(&self) -> Option<String> {
        unsafe { CefString::from_userfree(self.0.get_fragment_html.unwrap()(self.as_ptr())).map(String::from) }
    }
    /// Return the base URL that the fragment came from. This value is used for
    /// resolving relative URLs and may be empty.
    pub fn get_fragment_base_url(&self) -> Option<String> {
        unsafe { CefString::from_userfree(self.0.get_fragment_base_url.unwrap()(self.as_ptr())).map(String::from) }
    }
    /// Return the name of the file being dragged out of the browser window.
    pub fn get_file_name(&self) -> Option<String> {
        unsafe { CefString::from_userfree(self.0.get_file_name.unwrap()(self.as_ptr())).map(String::from) }
    }
    /// Write the contents of the file being dragged out of the web view into
    /// `writer`. Returns the number of bytes sent to `writer`. If `writer` is
    /// `None` this method will return the size of the file contents in bytes.
    /// Call [`get_file_name`] to get a suggested name for the file.
    pub fn get_file_contents(&self, writer: Option<StreamWriter>) -> usize {
        unsafe { self.0.get_file_contents.unwrap()(self.as_ptr(), writer.map(|w| w.into_raw()).unwrap_or(ptr::null_mut())) }
    }
    /// Retrieve the list of file names that are being dragged into the browser
    /// window.
    pub fn get_file_names(&self, names: &mut Vec<String>) -> bool {
        let mut string_list = CefStringList::new();
        let result = unsafe { self.0.get_file_names.unwrap()(self.as_ptr(), string_list.as_mut_ptr()) };
        names.extend(string_list.into_iter().map(String::from));
        result != 0
    }
    /// Set the link URL that is being dragged.
    pub fn set_link_url(&self, url: &str) {
        let url = CefString::new(url);
        unsafe{ self.0.set_link_url.unwrap()(self.as_ptr(), url.as_ptr()) };
    }
    /// Set the title associated with the link being dragged.
    pub fn set_link_title(&self, title: &str) {
        let title = CefString::new(title);
        unsafe{ self.0.set_link_title.unwrap()(self.as_ptr(), title.as_ptr()) };
    }
    /// Set the metadata associated with the link being dragged.
    pub fn set_link_metadata(&self, data: &str) {
        let data = CefString::new(data);
        unsafe{ self.0.set_link_metadata.unwrap()(self.as_ptr(), data.as_ptr()) };
    }
    /// Set the plain text fragment that is being dragged.
    pub fn set_fragment_text(&self, text: &str) {
        let text = CefString::new(text);
        unsafe{ self.0.set_fragment_text.unwrap()(self.as_ptr(), text.as_ptr()) };
    }
    /// Set the text/html fragment that is being dragged.
    pub fn set_fragment_html(&self, html: &str) {
        let html = CefString::new(html);
        unsafe{ self.0.set_fragment_html.unwrap()(self.as_ptr(), html.as_ptr()) };
    }
    /// Set the base URL that the fragment came from.
    pub fn set_fragment_base_url(&self, base_url: &str) {
        let base_url = CefString::new(base_url);
        unsafe{ self.0.set_fragment_base_url.unwrap()(self.as_ptr(), base_url.as_ptr()) };
    }
    /// Reset the file contents. You should do this before calling
    /// [`CefBrowserHost::DragTargetDragEnter`] as the web view does not allow us to
    /// drag in this kind of data.
    pub fn reset_file_contents(&self) {
        unsafe { self.0.reset_file_contents.unwrap()(self.as_ptr()) };
    }
    /// Add a file that is being dragged into the webview.
    pub fn add_file(&self, path: &str, display_name: &str) {
        let path = CefString::new(path);
        let display_name = CefString::new(display_name);
        unsafe{ self.0.add_file.unwrap()(self.as_ptr(), path.as_ptr(), display_name.as_ptr()) };
    }
    /// Get the image representation of drag data. May return `None` if no image
    /// representation is available.
    pub fn get_image(&self) -> Option<Image> {
        unsafe { Image::from_ptr(self.0.get_image.unwrap()(self.as_ptr())) }
    }
    /// Get the image hotspot (drag start location relative to image dimensions).
    pub fn get_image_hotspot(&self) -> Point {
        unsafe { self.0.get_image_hotspot.unwrap()(self.as_ptr()).into() }
    }
    /// Returns true if an image representation of drag data is available.
    pub fn has_image(&self) -> bool {
        unsafe { self.0.has_image.unwrap()(self.as_ptr()) != 0 }
    }
}

impl crate::helper_traits::DeepClone for DragData {
    /// Returns a writable copy of this object.
    fn deep_clone(&self) -> DragData {
        unsafe { Self::from_ptr_unchecked(self.0.clone.unwrap()(self.as_ptr())) }
    }
}

impl Default for DragData {
    fn default() -> Self {
        Self::new()
    }
}
