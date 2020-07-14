use cef_sys::cef_registration_t;

ref_counted_ptr!{
    /// Generic callback structure used for managing the lifespan of a registration.
    pub struct Registration(*mut cef_registration_t);
}
