use cef_sys::{
    cef_base_ref_counted_t, cef_dom_node_type_t, cef_domdocument_t, cef_domnode_t, cef_domvisitor_t,
};
use num_enum::UnsafeFromPrimitive;
use std::{collections::HashMap, convert::TryFrom};

use crate::{
    refcounted::{RefCounted, RefCounter},
    values::Rect,
};

/// DOM node types.
#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, UnsafeFromPrimitive)]
pub enum DOMNodeType {
    Unsupported = cef_dom_node_type_t::DOM_NODE_TYPE_UNSUPPORTED as i32,
    Element = cef_dom_node_type_t::DOM_NODE_TYPE_ELEMENT as i32,
    Attribute = cef_dom_node_type_t::DOM_NODE_TYPE_ATTRIBUTE as i32,
    Text = cef_dom_node_type_t::DOM_NODE_TYPE_TEXT as i32,
    CDataSection = cef_dom_node_type_t::DOM_NODE_TYPE_CDATA_SECTION as i32,
    ProcessingInstructions = cef_dom_node_type_t::DOM_NODE_TYPE_PROCESSING_INSTRUCTIONS as i32,
    Comment = cef_dom_node_type_t::DOM_NODE_TYPE_COMMENT as i32,
    Document = cef_dom_node_type_t::DOM_NODE_TYPE_DOCUMENT as i32,
    DocumentType = cef_dom_node_type_t::DOM_NODE_TYPE_DOCUMENT_TYPE as i32,
    DocumentFragment = cef_dom_node_type_t::DOM_NODE_TYPE_DOCUMENT_FRAGMENT as i32,
}

ref_counted_ptr!{
    /// Structure used to represent a DOM node. The functions of this structure
    /// should only be called on the render process main thread.
    pub struct DOMNode(*mut cef_domnode_t);
}

impl DOMNode {
    /// Returns the type for this node.
    pub fn get_type(&self) -> DOMNodeType {
        unimplemented!()
    }
    /// Returns true if this is a text node.
    pub fn is_text(&self) -> bool {
        unimplemented!()
    }
    /// Returns true if this is an element node.
    pub fn is_element(&self) -> bool {
        unimplemented!()
    }
    /// Returns true if this is an editable node.
    pub fn is_editable(&self) -> bool {
        unimplemented!()
    }
    /// Returns true if this is a form control element node.
    pub fn is_form_control_element(&self) -> bool {
        unimplemented!()
    }
    /// Returns the type of this form control element node.
    pub fn get_form_control_element_type(&self) -> String {
        unimplemented!()
    }
    /// Returns the name of this node.
    pub fn get_name(&self) -> String {
        unimplemented!()
    }
    /// Returns the value of this node.
    pub fn get_value(&self) -> String {
        unimplemented!()
    }
    /// Set the value of this node. Returns true on success.
    pub fn set_value(&mut self, value: &str) -> bool {
        unimplemented!()
    }
    /// Returns the contents of this node as markup.
    pub fn get_as_markup(&self) -> String {
        unimplemented!()
    }
    /// Returns the document associated with this node.
    pub fn get_document(&self) -> DOMDocument {
        unimplemented!()
    }
    /// Returns the parent node.
    pub fn get_parent(&self) -> Option<Self> {
        unimplemented!()
    }
    /// Returns the previous sibling node.
    pub fn get_previous_sibling(&self) -> Option<Self> {
        unimplemented!()
    }
    /// Returns the next sibling node.
    pub fn get_next_sibling(&self) -> Option<Self> {
        unimplemented!()
    }
    /// Returns true if this node has child nodes.
    pub fn has_children(&self) -> bool {
        unimplemented!()
    }
    /// Return the first child node.
    pub fn get_first_child(&self) -> Option<Self> {
        unimplemented!()
    }
    /// Returns the last child node.
    pub fn get_last_child(&self) -> Option<Self> {
        unimplemented!()
    }

    /// The following functions are valid only for element nodes.

    /// Returns the tag name of this element.
    pub fn get_element_tag_name(&self) -> Option<String> {
        unimplemented!()
    }
    /// Returns true if this element has attributes.
    pub fn has_element_attributes(&self) -> bool {
        unimplemented!()
    }
    /// Returns true if this element has an attribute named `attrName`.
    pub fn has_element_attribute(&self, attr_name: &str) -> bool {
        unimplemented!()
    }
    /// Returns the element attribute named |attrName|.
    pub fn get_element_attribute(&self, attr_name: &str) -> Option<String> {
        unimplemented!()
    }
    /// Returns a map of all element attributes.
    pub fn get_element_attributes(&self) -> HashMap<String, Option<String>> {
        unimplemented!()
    }
    /// Set the value for the element attribute named `attr_name`. Returns true
    /// on success.
    pub fn set_element_attribute(&mut self, attr_name: &str, value: &str) -> bool {
        unimplemented!()
    }
    /// Returns the inner text of the element.
    pub fn get_element_inner_text(&self) -> Option<String> {
        unimplemented!()
    }
    /// Returns the bounds of the element.
    pub fn get_element_bounds(&self) -> Option<Rect> {
        unimplemented!()
    }
}

impl PartialEq for DOMNode {
    /// Returns true if this object is pointing to the same handle as `that`
    /// object.
    fn eq(&self, that: &Self) -> bool {
        unimplemented!()
    }
}

ref_counted_ptr!{
    pub struct DOMDocument(*mut cef_domdocument_t);
}

/// Structure to implement for visiting the DOM. The functions of this structure
/// will be called on the render process main thread.
pub trait DOMVisitor: Send + Sync {
    /// Method executed for visiting the DOM. The document object passed to this
    /// function represents a snapshot of the DOM at the time this function is
    /// executed.
    fn visit(&self, document: &DOMDocument);
}

pub(crate) struct DOMVisitorWrapper;

impl DOMVisitorWrapper {
    pub(crate) fn wrap(delegate: Box<dyn DOMVisitor>) -> *mut cef_domvisitor_t {
        let mut rc = RefCounted::new(
            cef_domvisitor_t {
                base: unsafe { std::mem::zeroed() },
                visit: Some(Self::visit),
            },
            delegate,
        );
        unsafe { &mut *rc }.get_cef()
    }

    extern "C" fn visit(self_: *mut cef_domvisitor_t, document: *mut cef_domdocument_t) {
        let mut this = unsafe { RefCounted::<cef_domvisitor_t>::make_temp(self_) };
        this.visit(unsafe{ &DOMDocument::from_ptr_unchecked(document) });
        // we're done here!
        RefCounted::<cef_domvisitor_t>::release(this.get_cef() as *mut cef_base_ref_counted_t);
    }
}
