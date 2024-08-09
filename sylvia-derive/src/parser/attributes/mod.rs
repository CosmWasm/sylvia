//! Module defining parsing of Sylvia attributes.
//! Every Sylvia attribute should be prefixed with `sv::`

use proc_macro_error::emit_error;
use syn::spanned::Spanned;
use syn::{Attribute, MetaList, PathSegment};

pub mod attr;
pub mod custom;
pub mod error;
pub mod messages;
pub mod msg;
pub mod override_entry_point;

pub use attr::{MsgAttrForwarding, VariantAttrForwarding};
pub use custom::Custom;
pub use error::ContractErrorAttr;
pub use messages::{ContractMessageAttr, Customs};
pub use msg::{MsgAttr, MsgType};
pub use override_entry_point::{FilteredOverrideEntryPoints, OverrideEntryPoint};

/// This struct represents all possible attributes that
/// are parsed and utilized by sylvia.
pub enum SylviaAttribute {
    Custom,
    Error,
    Messages,
    Msg,
    OverrideEntryPoint,
    VariantAttrs,
    MsgAttrs,
}

impl SylviaAttribute {
    pub fn new(attr: &Attribute) -> Option<Self> {
        let segments = &attr.path().segments;
        if segments.len() == 2 && segments[0].ident == "sv" {
            Self::match_attribute(&segments[1])
        } else {
            None
        }
    }

    fn match_attribute(segment: &PathSegment) -> Option<Self> {
        match segment.ident.to_string().as_str() {
            "custom" => Some(Self::Custom),
            "error" => Some(Self::Error),
            "messages" => Some(Self::Messages),
            "msg" => Some(Self::Msg),
            "override_entry_point" => Some(Self::OverrideEntryPoint),
            "attr" => Some(Self::VariantAttrs),
            "msg_attr" => Some(Self::MsgAttrs),
            _ => None,
        }
    }
}

/// The structure parses all attributes provided in `new` method
/// and stores the one relevant for sylvia.
#[derive(Default)]
pub struct ParsedSylviaAttributes {
    pub custom_attr: Option<Custom>,
    pub error_attrs: Option<ContractErrorAttr>,
    pub messages_attrs: Vec<ContractMessageAttr>,
    pub msg_attr: Option<MsgAttr>,
    pub override_entry_point_attrs: Vec<OverrideEntryPoint>,
    pub variant_attrs_forward: Vec<VariantAttrForwarding>,
    pub msg_attrs_forward: Vec<MsgAttrForwarding>,
}

impl ParsedSylviaAttributes {
    pub fn new<'a>(attrs: impl Iterator<Item = &'a Attribute>) -> Self {
        let mut result = Self::default();
        for attr in attrs {
            let sylvia_attr = SylviaAttribute::new(attr);
            let attr_content = attr.meta.require_list();

            if let (Some(sylvia_attr), Ok(attr)) = (sylvia_attr, &attr_content) {
                result.match_attribute(&sylvia_attr, attr);
            }
        }

        if let Some(attr) = result.variant_attrs_forward.first() {
            if let Some(MsgAttr::Instantiate) = result.msg_attr {
                emit_error!(
                    attr.span, "The attribute `sv::attr` is not supported for `instantiate`";
                    note = "Message `instantiate` is a structure, use `#[sv::msg_attr] instead`";
                );
            } else if let Some(MsgAttr::Migrate) = result.msg_attr {
                emit_error!(
                    attr.span, "The attribute `sv::attr` is not supported for `migrate`";
                    note = "Message `migrate` is a structure, use `#[sv::msg_attr] instead`";
                );
            }
        }

        result
    }

    fn match_attribute(&mut self, attribute_type: &SylviaAttribute, attr: &MetaList) {
        match attribute_type {
            SylviaAttribute::Custom => {
                if self.custom_attr.is_none() {
                    if let Ok(custom_attr) = Custom::new(attr) {
                        self.custom_attr = Some(custom_attr);
                    }
                } else {
                    emit_error!(
                        attr, "The attribute `sv::custom` is redefined";
                        note = attr.span() => "Previous definition of the attribute `sv::custom`";
                        note = "Only one `sv::custom` attribute can exist on a single sylvia entity"
                    );
                }
            }
            SylviaAttribute::Error => {
                if self.error_attrs.is_none() {
                    if let Ok(error_attr) = ContractErrorAttr::new(attr) {
                        self.error_attrs = Some(error_attr);
                    }
                } else {
                    emit_error!(
                        attr, "The attribute `sv::error` is redefined";
                        note = attr.span() => "Previous definition of the attribute `sv::error`";
                        note = "Only one `sv::error` attribute can exist on a single method"
                    );
                }
            }
            SylviaAttribute::Messages => {
                if let Ok(contract) = ContractMessageAttr::new(attr) {
                    self.messages_attrs.push(contract);
                }
            }
            SylviaAttribute::Msg => {
                if self.msg_attr.is_none() {
                    if let Ok(msg_attr) = MsgAttr::new(attr) {
                        self.msg_attr = Some(msg_attr);
                    }
                } else {
                    emit_error!(
                        attr, "The attribute `sv::msg` is redefined";
                        note = attr.span() => "Previous definition of the attribute `sv::msg`";
                        note = "Only one `sv::msg` attribute can exist on a single method"
                    );
                }
            }
            SylviaAttribute::OverrideEntryPoint => {
                if let Ok(override_entry_point) = OverrideEntryPoint::new(attr) {
                    self.override_entry_point_attrs.push(override_entry_point)
                }
            }
            SylviaAttribute::VariantAttrs => {
                self.variant_attrs_forward
                    .push(VariantAttrForwarding::new(attr));
            }
            SylviaAttribute::MsgAttrs => {
                if let Ok(message_attrs) = MsgAttrForwarding::new(attr) {
                    self.msg_attrs_forward.push(message_attrs);
                }
            }
        }
    }
}
