use cef_sys::{_cef_domnode_t};
use std::convert::TryFrom;

pub struct DOMNode(*mut _cef_domnode_t);

impl TryFrom<*mut _cef_domnode_t> for DOMNode {
    type Error = ();

    fn try_from(node: *mut _cef_domnode_t) -> Result<Self, Self::Error> {
        if node.is_null() {
            Err(())
        } else {
            unsafe { ((*node).base.add_ref.unwrap())(&mut (*node).base); }
            Ok(Self(node))
        }
    }
}

impl Drop for DOMNode {
    fn drop(&mut self) {
        unsafe { ((*self.0).base.release.unwrap())(&mut (*self.0).base); }
    }
}
