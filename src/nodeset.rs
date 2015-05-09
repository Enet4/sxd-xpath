use std::borrow::ToOwned;
use std::collections::HashMap;
use std::iter::{IntoIterator,FromIterator};
use std::{slice,vec};

use document::QName;
use document::dom4;

use super::EvaluationContext;

macro_rules! unpack(
    ($enum_name:ident, $name:ident, $wrapper:ident, dom4::$inner:ident) => (
        impl<'d> $enum_name<'d> {
            pub fn $name(self) -> Option<dom4::$inner<'d>> {
                match self {
                    $enum_name::$wrapper(n) => Some(n),
                    _ => None,
                }
            }
        }
    )
);

macro_rules! conversion_trait(
    ($res_type:ident, {
        $(dom4::$leaf_type:ident => Node::$variant:ident),*
    }) => (
        $(impl<'d> From<dom4::$leaf_type<'d>> for $res_type<'d>  {
            fn from(v: dom4::$leaf_type<'d>) -> $res_type<'d> {
                Node::$variant(v)
            }
        })*
    )
);

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub struct Namespace<'d> {
    pub parent: dom4::Element<'d>,
    pub prefix: &'d str,
    pub uri: &'d str,
}

impl<'d> Namespace<'d> {
    pub fn document(&self) -> &'d dom4::Document<'d> { self.parent.document() }
    pub fn parent(&self) -> dom4::Element<'d> { self.parent }
    pub fn prefix(&self) -> &'d str { self.prefix }
    pub fn uri(&self) -> &'d str { self.uri }
    pub fn expanded_name(&self) -> QName<'d> { QName::new(self.prefix) }
}

#[derive(Copy,Clone,PartialEq,Eq,Hash,Debug)]
pub enum Node<'d> {
    Root(dom4::Root<'d>),
    Element(dom4::Element<'d>),
    Attribute(dom4::Attribute<'d>),
    Text(dom4::Text<'d>),
    Comment(dom4::Comment<'d>),
    Namespace(Namespace<'d>),
    ProcessingInstruction(dom4::ProcessingInstruction<'d>),
}

unpack!(Node, root, Root, dom4::Root);
unpack!(Node, element, Element, dom4::Element);
unpack!(Node, attribute, Attribute, dom4::Attribute);
unpack!(Node, text, Text, dom4::Text);
unpack!(Node, comment, Comment, dom4::Comment);
unpack!(Node, processing_instruction, ProcessingInstruction, dom4::ProcessingInstruction);

impl<'d> Node<'d> {
    pub fn document(&self) -> &'d dom4::Document<'d> {
        use self::Node::*;
        match *self {
            Root(n)                  => n.document(),
            Element(n)               => n.document(),
            Attribute(n)             => n.document(),
            Text(n)                  => n.document(),
            Comment(n)               => n.document(),
            ProcessingInstruction(n) => n.document(),
            Namespace(n)             => n.document(),
        }
    }

    pub fn prefixed_name(&self) -> Option<String> {
        use self::Node::*;

        fn qname_prefixed_name(element: dom4::Element, name: QName, preferred_prefix: Option<&str>) -> String {
            if let Some(ns_uri) = name.namespace_uri() {
                if let Some(prefix) = element.prefix_for_namespace_uri(ns_uri, preferred_prefix) {
                    format!("{}:{}", prefix, name.local_part())
                } else {
                    name.local_part().to_owned()
                }
            } else {
                name.local_part().to_owned()
            }
        };

        match *self {
            Root(_)                  => None,
            Element(n)               => {
                Some(qname_prefixed_name(n, n.name(), n.preferred_prefix()))
            },
            Attribute(n)             => {
                let parent = n.parent().expect("Cannot process attribute without parent");
                Some(qname_prefixed_name(parent, n.name(), n.preferred_prefix()))
            },
            Text(_)                  => None,
            Comment(_)               => None,
            ProcessingInstruction(n) => Some(n.target().to_owned()),
            Namespace(n)             => Some(n.prefix().to_owned()),
        }
    }

    pub fn expanded_name(&self) -> Option<QName<'d>> {
        use self::Node::*;
        match *self {
            Root(_)                  => None,
            Element(n)               => Some(n.name()),
            Attribute(n)             => Some(n.name()),
            Text(_)                  => None,
            Comment(_)               => None,
            ProcessingInstruction(n) => Some(QName::new(n.target())),
            Namespace(n)             => Some(n.expanded_name())
        }
    }

    pub fn parent(&self) -> Option<Node<'d>> {
        use self::Node::*;
        match *self {
            Root(_)                  => None,
            Element(n)               => n.parent().map(|n| n.into()),
            Attribute(n)             => n.parent().map(|n| n.into()),
            Text(n)                  => n.parent().map(|n| n.into()),
            Comment(n)               => n.parent().map(|n| n.into()),
            ProcessingInstruction(n) => n.parent().map(|n| n.into()),
            Namespace(n)             => Some(n.parent().into()),
        }
    }

    pub fn children(&self) -> Vec<Node<'d>> {
        use self::Node::*;
        match *self {
            Root(n)                  => n.children().iter().map(|&n| n.into()).collect(),
            Element(n)               => n.children().iter().map(|&n| n.into()).collect(),
            Attribute(_)             => Vec::new(),
            Text(_)                  => Vec::new(),
            Comment(_)               => Vec::new(),
            ProcessingInstruction(_) => Vec::new(),
            Namespace(_)             => Vec::new(),
        }
    }

    pub fn preceding_siblings(&self) -> Vec<Node<'d>> {
        use self::Node::*;
        match *self {
            Root(_)                  => Vec::new(),
            Element(n)               => n.preceding_siblings().iter().rev().map(|&n| n.into()).collect(),
            Attribute(_)             => Vec::new(),
            Text(n)                  => n.preceding_siblings().iter().rev().map(|&n| n.into()).collect(),
            Comment(n)               => n.preceding_siblings().iter().rev().map(|&n| n.into()).collect(),
            ProcessingInstruction(n) => n.preceding_siblings().iter().rev().map(|&n| n.into()).collect(),
            Namespace(_)             => Vec::new(),
        }
    }

    pub fn following_siblings(&self) -> Vec<Node<'d>> {
        use self::Node::*;
        match *self {
            Root(_)                  => Vec::new(),
            Element(n)               => n.following_siblings().iter().map(|&n| n.into()).collect(),
            Attribute(_)             => Vec::new(),
            Text(n)                  => n.following_siblings().iter().map(|&n| n.into()).collect(),
            Comment(n)               => n.following_siblings().iter().map(|&n| n.into()).collect(),
            ProcessingInstruction(n) => n.following_siblings().iter().map(|&n| n.into()).collect(),
            Namespace(_)             => Vec::new(),
        }
    }

    pub fn string_value(&self) -> String {
        use self::Node::*;

        fn document_order_text_nodes(node: &Node, result: &mut String) {
            for child in node.children().iter() {
                match child {
                    &Node::Element(_) => document_order_text_nodes(child, result),
                    &Node::Text(n) => result.push_str(n.text()),
                    _ => {},
                }
            }
        };

        fn text_descendants_string_value(node: &Node) -> String {
            let mut result = String::new();
            document_order_text_nodes(node, &mut result);
            result
        }

        match *self {
            Root(_)                  => text_descendants_string_value(self),
            Element(_)               => text_descendants_string_value(self),
            Attribute(n)             => n.value().to_owned(),
            ProcessingInstruction(n) => n.value().unwrap_or("").to_owned(),
            Comment(n)               => n.text().to_owned(),
            Text(n)                  => n.text().to_owned(),
            Namespace(n)             => n.uri().to_owned(),
        }
    }
}

conversion_trait!(Node, {
    dom4::Root                  => Node::Root,
    dom4::Element               => Node::Element,
    dom4::Attribute             => Node::Attribute,
    dom4::Text                  => Node::Text,
    dom4::Comment               => Node::Comment,
    dom4::ProcessingInstruction => Node::ProcessingInstruction
});

impl<'d> Into<Node<'d>> for dom4::ChildOfRoot<'d> {
    fn into(self) -> Node<'d> {
        use self::Node::*;
        match self {
            dom4::ChildOfRoot::Element(n)               => Element(n),
            dom4::ChildOfRoot::Comment(n)               => Comment(n),
            dom4::ChildOfRoot::ProcessingInstruction(n) => ProcessingInstruction(n),
        }
    }
}

impl<'d> Into<Node<'d>> for dom4::ChildOfElement<'d> {
    fn into(self) -> Node<'d> {
        use self::Node::*;
        match self {
            dom4::ChildOfElement::Element(n)               => Element(n),
            dom4::ChildOfElement::Text(n)                  => Text(n),
            dom4::ChildOfElement::Comment(n)               => Comment(n),
            dom4::ChildOfElement::ProcessingInstruction(n) => ProcessingInstruction(n),
        }
    }
}

impl<'d> Into<Node<'d>> for dom4::ParentOfChild<'d> {
    fn into(self) -> Node<'d> {
        use self::Node::*;
        match self {
            dom4::ParentOfChild::Root(n)    => Root(n),
            dom4::ParentOfChild::Element(n) => Element(n),
        }
    }
}

/// A collection of nodes
#[derive(PartialEq,Debug,Clone)]
pub struct Nodeset<'d> {
    nodes: Vec<Node<'d>>,
}

impl<'d> Nodeset<'d> {
    pub fn new() -> Nodeset<'d> {
        Nodeset { nodes: Vec::new() }
    }

    pub fn add<N>(&mut self, node: N)
        where N: Into<Node<'d>>
    {
        self.nodes.push(node.into());
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, 'd> {
        IntoIterator::into_iter(self)
    }

    pub fn into_iter(self) -> IntoIter<'d> {
        IntoIterator::into_iter(self)
    }

    pub fn add_nodeset(& mut self, other: &Nodeset<'d>) {
        self.nodes.push_all(&other.nodes);
    }

    pub fn size(&self) -> usize {
        self.nodes.len()
    }

    pub fn document_order_first(&self) -> Option<Node<'d>> {
        let doc = match self.nodes.first() {
            Some(n) => n.document(),
            None => return None,
        };

        let mut idx = 0;
        let mut stack = vec![doc.root().into()];
        let mut order: HashMap<Node, usize> = HashMap::new();

        // Rebuilding this each time cannot possibly be performant,
        // but I want to see how widely used this is first before
        // picking an appropriate caching point.

        while let Some(n) = stack.pop() {
            order.insert(n, idx);
            idx += 1;
            let c = n.children();

            stack.extend(c.into_iter().rev());

            if let Node::Element(e) = n {
                // TODO: namespaces
                stack.extend(e.attributes().into_iter().map(|a| a.into()));
            }
        }

        self.nodes.iter().min_by(|&n| order[n]).map(|n| *n)
    }
}

impl<'a, 'd : 'a> IntoIterator for &'a Nodeset<'d> {
    type Item = Node<'d>;
    type IntoIter = Iter<'a, 'd>;

    fn into_iter(self) -> Iter<'a, 'd> {
        Iter { iter: self.nodes.iter() }
    }
}

impl<'d> IntoIterator for Nodeset<'d> {
    type Item = Node<'d>;
    type IntoIter = IntoIter<'d>;

    fn into_iter(self) -> IntoIter<'d> {
        IntoIter { iter: self.nodes.into_iter() }
    }
}

impl<'a, 'd : 'a> FromIterator<EvaluationContext<'a, 'd>> for Nodeset<'d> {
    fn from_iter<T>(iterator: T) -> Nodeset<'d>
        where T: IntoIterator<Item=EvaluationContext<'a, 'd>>
    {
        let mut ns = Nodeset::new();
        for n in iterator { ns.add(n.node) };
        ns
    }
}

pub struct Iter<'a, 'd : 'a> {
    iter: slice::Iter<'a, Node<'d>>,
}

impl<'a, 'd : 'a> Iterator for Iter<'a, 'd> {
    type Item = Node<'d>;
    fn next(&mut self) -> Option<Node<'d>> { self.iter.next().cloned() }
}

pub struct IntoIter<'d> {
    iter: vec::IntoIter<Node<'d>>,
}

impl<'d> Iterator for IntoIter<'d> {
    type Item = Node<'d>;
    fn next(&mut self) -> Option<Node<'d>> { self.iter.next() }
}

#[cfg(test)]
mod test {
    use std::borrow::ToOwned;

    use document::Package;

    use super::Node::{
        Attribute,
        Comment,
        Element,
        ProcessingInstruction,
        Root,
        Text,
    };
    use super::{Nodeset,Node};

    fn into_node<'d, T: Into<Node<'d>>>(n: T) -> Node<'d> { n.into() }

    #[test]
    fn nodeset_can_include_all_node_types() {
        let package = Package::new();
        let doc = package.as_document();
        let mut nodes = Nodeset::new();

        let r = doc.root();
        let e = doc.create_element("element");
        let a = e.set_attribute_value("name", "value");
        let t = doc.create_text("text");
        let c = doc.create_comment("comment");
        let p = doc.create_processing_instruction("pi", None);

        nodes.add(r);
        nodes.add(e);
        nodes.add(a);
        nodes.add(t);
        nodes.add(c);
        nodes.add(p);

        let node_vec: Vec<_> = nodes.iter().collect();

        assert_eq!(6, node_vec.len());
        assert_eq!(node_vec[0], Root(r));
        assert_eq!(node_vec[1], Element(e));
        assert_eq!(node_vec[2], Attribute(a));
        assert_eq!(node_vec[3], Text(t));
        assert_eq!(node_vec[4], Comment(c));
        assert_eq!(node_vec[5], ProcessingInstruction(p));
    }

    #[test]
    fn nodesets_can_be_combined() {
        let package = Package::new();
        let doc = package.as_document();

        let mut all_nodes = Nodeset::new();
        let mut nodes1 = Nodeset::new();
        let mut nodes2 = Nodeset::new();

        let e1 = doc.create_element("element1");
        let e2 = doc.create_element("element2");

        all_nodes.add(e1);
        all_nodes.add(e2);

        nodes1.add(e1);
        nodes2.add(e2);

        nodes1.add_nodeset(&nodes2);

        assert_eq!(all_nodes, nodes1);
    }

    #[test]
    fn nodeset_knows_first_node_in_document_order() {
        let package = Package::new();
        let doc = package.as_document();

        let c1 = doc.create_comment("1");
        let c2 = doc.create_comment("2");
        doc.root().append_child(c1);
        doc.root().append_child(c2);

        let nodes = nodeset![c2, c1];

        assert_eq!(Some(into_node(c1)), nodes.document_order_first());
    }

    #[test]
    fn attributes_come_before_children_in_document_order() {
        let package = Package::new();
        let doc = package.as_document();

        let parent = doc.create_element("parent");
        let attr = parent.set_attribute_value("a", "v");
        let child = doc.create_element("child");

        doc.root().append_child(parent);
        parent.append_child(child);

        let nodes = nodeset![child, attr];

        assert_eq!(Some(attr.into()), nodes.document_order_first());
    }

    #[test]
    fn prefixed_name_of_element_with_preferred_prefix() {
        let package = Package::new();
        let doc = package.as_document();

        let e = doc.create_element(("uri", "wow"));
        e.set_preferred_prefix(Some("prefix"));
        e.register_prefix("prefix", "uri");
        let node: Node = e.into();

        assert_eq!(Some("prefix:wow".to_owned()), node.prefixed_name());
    }

    #[test]
    fn prefixed_name_of_element_with_prefix() {
        let package = Package::new();
        let doc = package.as_document();

        let e = doc.create_element(("uri", "wow"));
        e.register_prefix("prefix", "uri");
        let node: Node = e.into();

        assert_eq!(Some("prefix:wow".to_owned()), node.prefixed_name());
    }

    #[test]
    fn prefixed_name_of_element_without_prefix() {
        // See library-level doc about missing prefixes
        let package = Package::new();
        let doc = package.as_document();

        let e = doc.create_element(("uri", "wow"));
        let node: Node = e.into();

        assert_eq!(Some("wow".to_owned()), node.prefixed_name());
    }

    #[test]
    fn prefixed_name_of_attribute_with_preferred_prefix() {
        let package = Package::new();
        let doc = package.as_document();

        let e = doc.create_element("element");
        let a = e.set_attribute_value(("uri", "attr"), "value");
        a.set_preferred_prefix(Some("prefix"));
        e.register_prefix("prefix", "uri");
        let node: Node = a.into();

        assert_eq!(Some("prefix:attr".to_owned()), node.prefixed_name());
    }

    #[test]
    fn prefixed_name_of_attribute_with_prefix() {
        let package = Package::new();
        let doc = package.as_document();

        let e = doc.create_element("element");
        let a = e.set_attribute_value(("uri", "attr"), "value");
        e.register_prefix("prefix", "uri");
        let node: Node = a.into();

        assert_eq!(Some("prefix:attr".to_owned()), node.prefixed_name());
    }

    #[test]
    fn prefixed_name_of_processing_instruction() {
        let package = Package::new();
        let doc = package.as_document();

        let pi = doc.create_processing_instruction("target", Some("value"));
        let node: Node = pi.into();

        assert_eq!(Some("target".to_owned()), node.prefixed_name());
    }

    #[test]
    fn string_value_of_element_node_is_concatenation_of_descendant_text_nodes() {
        let package = Package::new();
        let doc = package.as_document();

        let element = doc.create_element("hello");
        let child = doc.create_element("world");
        let text1 = doc.create_text("Presenting: ");
        let text2 = doc.create_text("Earth");
        let text3 = doc.create_text("!");

        element.append_child(text1);
        element.append_child(child);
        child.append_child(text2);
        element.append_child(text3);

        assert_eq!("Presenting: Earth!", into_node(element).string_value());
    }

    #[test]
    fn string_value_of_attribute_node_is_value() {
        let package = Package::new();
        let doc = package.as_document();
        let element = doc.create_element("hello");
        let attribute: Node = element.set_attribute_value("world", "Earth").into();
        assert_eq!("Earth", attribute.string_value());
    }

    #[test]
    fn string_value_of_pi_node_is_empty_when_no_value() {
        let package = Package::new();
        let doc = package.as_document();
        let pi: Node = doc.create_processing_instruction("hello", None).into();
        assert_eq!("", pi.string_value());
    }

    #[test]
    fn string_value_of_pi_node_is_the_value_when_value() {
        let package = Package::new();
        let doc = package.as_document();
        let pi: Node = doc.create_processing_instruction("hello", Some("world")).into();
        assert_eq!("world", pi.string_value());
    }

    #[test]
    fn string_value_of_comment_node_is_the_text() {
        let package = Package::new();
        let doc = package.as_document();
        let comment: Node = doc.create_comment("hello world").into();
        assert_eq!("hello world", comment.string_value());
    }

    #[test]
    fn string_value_of_text_node_is_the_text() {
        let package = Package::new();
        let doc = package.as_document();
        let text: Node = doc.create_text("hello world").into();
        assert_eq!("hello world", text.string_value());
    }
}
