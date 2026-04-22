use dioxus::core::DynamicNode;
use dioxus::prelude::*;

pub trait ElementExt {
    fn has_content(&self) -> bool;
}

impl ElementExt for Element {
    fn has_content(&self) -> bool {
        self.as_ref().map_or(false, |node| {
            !matches!(node.dynamic_root(0), Some(DynamicNode::Placeholder(_)))
        })
    }
}
