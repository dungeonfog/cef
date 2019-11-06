use cef_sys::{cef_dom_node_type_t, cef_domdocument_t, cef_domnode_t, cef_domvisitor_t};
use num_enum::UnsafeFromPrimitive;
use std::{collections::HashMap};
use parking_lot::Mutex;

use crate::{
    refcounted::{RefCountedPtr, Wrapper},
    string::CefString,
    values::Rect,
};

/// DOM node types.
#[repr(i32)]
#[derive(Copy, Clone, PartialEq, Eq, UnsafeFromPrimitive)]
pub enum DOMNodeType {
    Unsupported = cef_dom_node_type_t::DOM_NODE_TYPE_UNSUPPORTED,
    Element = cef_dom_node_type_t::DOM_NODE_TYPE_ELEMENT,
    Attribute = cef_dom_node_type_t::DOM_NODE_TYPE_ATTRIBUTE,
    Text = cef_dom_node_type_t::DOM_NODE_TYPE_TEXT,
    CDataSection = cef_dom_node_type_t::DOM_NODE_TYPE_CDATA_SECTION,
    ProcessingInstructions = cef_dom_node_type_t::DOM_NODE_TYPE_PROCESSING_INSTRUCTIONS,
    Comment = cef_dom_node_type_t::DOM_NODE_TYPE_COMMENT,
    Document = cef_dom_node_type_t::DOM_NODE_TYPE_DOCUMENT,
    DocumentType = cef_dom_node_type_t::DOM_NODE_TYPE_DOCUMENT_TYPE,
    DocumentFragment = cef_dom_node_type_t::DOM_NODE_TYPE_DOCUMENT_FRAGMENT,
}

ref_counted_ptr! {
    /// Structure used to represent a DOM node. The functions of this structure
    /// should only be called on the render process main thread.
    pub struct DOMNode(*mut cef_domnode_t);
}

impl DOMNode {
    /// Returns the type for this node.
    pub fn get_type(&self) -> DOMNodeType {
        unsafe{ DOMNodeType::from_unchecked((self.0.get_type.unwrap())(self.as_ptr())) }
    }
    /// Returns true if this is a text node.
    pub fn is_text(&self) -> bool {
        unsafe{ (self.0.is_text.unwrap())(self.as_ptr()) != 0 }
    }
    /// Returns true if this is an element node.
    pub fn is_element(&self) -> bool {
        unsafe{ (self.0.is_element.unwrap())(self.as_ptr()) != 0 }
    }
    /// Returns true if this is an editable node.
    pub fn is_editable(&self) -> bool {
        unsafe{ (self.0.is_editable.unwrap())(self.as_ptr()) != 0 }
    }
    /// Returns true if this is a form control element node.
    pub fn is_form_control_element(&self) -> bool {
        unsafe{ (self.0.is_form_control_element.unwrap())(self.as_ptr()) != 0 }
    }
    /// Returns the type of this form control element node.
    pub fn get_form_control_element_type(&self) -> String {
        unsafe{ CefString::from_userfree_unchecked((self.0.get_form_control_element_type.unwrap())(self.as_ptr())).into() }
    }
    /// Returns the name of this node.
    pub fn get_name(&self) -> String {
        unsafe{ CefString::from_userfree_unchecked((self.0.get_name.unwrap())(self.as_ptr())).into() }
    }
    /// Returns the value of this node.
    pub fn get_value(&self) -> String {
        unsafe{ CefString::from_userfree_unchecked((self.0.get_value.unwrap())(self.as_ptr())).into() }
    }
    /// Set the value of this node. Returns true on success.
    pub fn set_value(&self, value: &str) -> bool {
        let value = CefString::new(value);
        unsafe{ (self.0.set_value.unwrap())(self.as_ptr(), value.as_ptr()) != 0 }
    }
    /// Returns the contents of this node as markup.
    pub fn get_as_markup(&self) -> String {
        unsafe{ CefString::from_userfree_unchecked((self.0.get_as_markup.unwrap())(self.as_ptr())).into() }
    }
    /// Returns the document associated with this node.
    pub fn get_document(&self) -> DOMDocument {
        unsafe{ DOMDocument::from_ptr_unchecked((self.0.get_document.unwrap())(self.as_ptr())) }
    }
    /// Returns the parent node.
    pub fn get_parent(&self) -> Option<Self> {
        unsafe{ DOMNode::from_ptr((self.0.get_parent.unwrap())(self.as_ptr())) }
    }
    /// Returns the previous sibling node.
    pub fn get_previous_sibling(&self) -> Option<Self> {
        unsafe{ DOMNode::from_ptr((self.0.get_previous_sibling.unwrap())(self.as_ptr())) }
    }
    /// Returns the next sibling node.
    pub fn get_next_sibling(&self) -> Option<Self> {
        unsafe{ DOMNode::from_ptr((self.0.get_next_sibling.unwrap())(self.as_ptr())) }
    }
    /// Returns true if this node has child nodes.
    pub fn has_children(&self) -> bool {
        unsafe{ (self.0.has_children.unwrap())(self.as_ptr()) != 0 }
    }
    /// Return the first child node.
    pub fn get_first_child(&self) -> Option<Self> {
        unsafe{ DOMNode::from_ptr((self.0.get_first_child.unwrap())(self.as_ptr())) }
    }
    /// Returns the last child node.
    pub fn get_last_child(&self) -> Option<Self> {
        unsafe{ DOMNode::from_ptr((self.0.get_last_child.unwrap())(self.as_ptr())) }
    }

    /// The following functions are valid only for element nodes.

    /// Returns the tag name of this element.
    pub fn get_element_tag_name(&self) -> Option<String> {
        unsafe{ CefString::from_userfree((self.0.get_element_tag_name.unwrap())(self.as_ptr())).map(String::from) }
    }
    /// Returns true if this element has attributes.
    pub fn has_element_attributes(&self) -> bool {
        unsafe{ (self.0.has_element_attributes.unwrap())(self.as_ptr()) != 0 }
    }
    /// Returns true if this element has an attribute named `attrName`.
    pub fn has_element_attribute(&self, attr_name: &str) -> bool {
        let attr_name = CefString::new(attr_name);
        unsafe{ (self.0.has_element_attribute.unwrap())(self.as_ptr(), attr_name.as_ptr()) != 0 }
    }
    /// Returns the element attribute named |attrName|.
    pub fn get_element_attribute(&self, attr_name: &str) -> Option<String> {
        let attr_name = CefString::new(attr_name);
        unsafe{ CefString::from_userfree(self.0.get_element_attribute.unwrap()(self.as_ptr(), attr_name.as_ptr())).map(String::from) }
    }
    /// Returns a map of all element attributes.
    pub fn get_element_attributes(&self) -> HashMap<String, Option<String>> {
        unimplemented!()
    }
    /// Set the value for the element attribute named `attr_name`. Returns true
    /// on success.
    pub fn set_element_attribute(&self, attr_name: &str, value: &str) -> bool {
        let attr_name = CefString::new(attr_name);
        let value = CefString::new(value);
        unsafe{ (self.0.set_element_attribute.unwrap())(self.as_ptr(), attr_name.as_ptr(), value.as_ptr()) != 0 }
    }
    /// Returns the inner text of the element.
    pub fn get_element_inner_text(&self) -> Option<String> {
        unsafe{ CefString::from_userfree((self.0.get_element_inner_text.unwrap())(self.as_ptr())).map(String::from) }
    }
    /// Returns the bounds of the element.
    pub fn get_element_bounds(&self) -> Rect {
        let rect = unsafe{ (self.0.get_element_bounds.unwrap())(self.as_ptr()) };
        Rect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        }
    }
}

impl PartialEq for DOMNode {
    /// Returns true if this object is pointing to the same handle as `that`
    /// object.
    fn eq(&self, that: &Self) -> bool {
        unimplemented!()
    }
}

ref_counted_ptr! {
    pub struct DOMDocument(*mut cef_domdocument_t);
}

ref_counted_ptr!{
    pub struct DOMVisitor(*mut cef_domvisitor_t);
}

impl DOMVisitor {
    pub fn new<C: DOMVisitorCallback>(callback: C) -> DOMVisitor {
        unsafe{ DOMVisitor::from_ptr_unchecked(DOMVisitorWrapper::new(Box::new(callback)).wrap().into_raw()) }
    }
}

/// Structure to implement for visiting the DOM. The functions of this structure
/// will be called on the render process main thread.
pub trait DOMVisitorCallback = 'static + Send + FnMut(DOMDocument);

pub(crate) struct DOMVisitorWrapper {
    delegate: Mutex<Box<dyn DOMVisitorCallback>>,
}

impl Wrapper for DOMVisitorWrapper {
    type Cef = cef_domvisitor_t;
    fn wrap(self) -> RefCountedPtr<Self::Cef> {
        RefCountedPtr::wrap(
            cef_domvisitor_t {
                base: unsafe { std::mem::zeroed() },
                visit: Some(Self::visit),
            },
            self,
        )
    }
}

impl DOMVisitorWrapper {
    pub(crate) fn new(delegate: Box<dyn DOMVisitorCallback>) -> DOMVisitorWrapper {
        DOMVisitorWrapper { delegate: Mutex::new(delegate) }
    }
}

cef_callback_impl! {
    impl for DOMVisitorWrapper: cef_domvisitor_t {
        fn visit(
            &self,
            document: DOMDocument: *mut cef_domdocument_t,
        ) {
            (&mut *self.delegate.lock())(document);
        }
    }
}
