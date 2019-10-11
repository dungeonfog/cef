use cef_sys::{
    cef_base_ref_counted_t, cef_drag_data_create, cef_drag_data_t, cef_drag_operations_mask_t,
};
use num_enum::UnsafeFromPrimitive;
use std::collections::HashSet;

/// "Verb" of a drag-and-drop operation as negotiated between the source and
/// destination. These constants match their equivalents in WebCore's
/// DragActions.h.
#[repr(i32)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, UnsafeFromPrimitive, Hash)]
pub enum DragOperation {
    None = cef_drag_operations_mask_t::DRAG_OPERATION_NONE.0,
    Copy = cef_drag_operations_mask_t::DRAG_OPERATION_COPY.0,
    Link = cef_drag_operations_mask_t::DRAG_OPERATION_LINK.0,
    Generic = cef_drag_operations_mask_t::DRAG_OPERATION_GENERIC.0,
    Private = cef_drag_operations_mask_t::DRAG_OPERATION_PRIVATE.0,
    Move = cef_drag_operations_mask_t::DRAG_OPERATION_MOVE.0,
    Delete = cef_drag_operations_mask_t::DRAG_OPERATION_DELETE.0,
}

impl DragOperation {
    pub fn every() -> HashSet<DragOperation> {
        [
            DragOperation::None,
            DragOperation::Copy,
            DragOperation::Link,
            DragOperation::Generic,
            DragOperation::Private,
            DragOperation::Move,
            DragOperation::Delete,
        ]
        .iter()
        .cloned()
        .collect()
    }
}

ref_counted_ptr!{
    /// Structure used to represent drag data. The functions of this structure may be
    /// called on any thread.
    pub struct DragData(*mut cef_drag_data_t);
}

unsafe impl Sync for DragData {}
unsafe impl Send for DragData {}

impl DragData {
    pub fn new() -> Self {
        unsafe{ Self::from_ptr_unchecked(cef_drag_data_create()) }
    }
}
