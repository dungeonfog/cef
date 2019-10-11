use cef_sys::cef_image_t;

ref_counted_ptr!{
    pub struct Image(*mut cef_image_t);
}
