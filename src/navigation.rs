use cef_sys::cef_navigation_entry_t;

ref_counted_ptr!{
    pub struct NavigationEntry(*mut cef_navigation_entry_t);
}
