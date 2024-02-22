use proc_macro_error::emit_error;
use syn::spanned::Spanned;
use syn::{Attribute, PathSegment};

pub mod custom;
pub mod error;
pub mod messages;
pub mod msg;
pub mod override_entry_point;

pub use custom::Custom;
pub use error::ContractErrorAttr;
pub use messages::{ContractMessageAttr, Customs};
pub use msg::{MsgAttr, MsgType};
pub use override_entry_point::{FilteredOverrideEntryPoints, OverrideEntryPoint};

/// This struct represents all possible attributes that
/// are parsed and utilized by sylvia.
///
pub enum SylviaAttribute {
    Custom,
    Error,
    Messages,
    Msg,
    OverrideEntryPoint,
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
            _ => None,
        }
    }
}

/// The structure parses all attributes provided in `new` method
/// and stores the one relevant for sylvia.
///
#[derive(Default)]
pub struct ParsedSylviaAttributes {
    pub custom_attr: Option<Custom>,
    pub error_attrs: Option<ContractErrorAttr>,
    pub messages_attrs: Vec<ContractMessageAttr>,
    pub msg_attr: Option<MsgAttr>,
    pub override_entry_point_attrs: Vec<OverrideEntryPoint>,
}

impl ParsedSylviaAttributes {
    pub fn new<'a>(attrs: impl Iterator<Item = &'a Attribute>) -> Self {
        let mut result = Self::default();
        for attr in attrs {
            let sylvia_attr = SylviaAttribute::new(attr);
            if let Some(sylvia_attr) = sylvia_attr {
                result.match_attribute(&sylvia_attr, attr);
            }
        }
        result
    }

    fn match_attribute(&mut self, attribute_type: &SylviaAttribute, attr: &Attribute) {
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
        }
    }
}
