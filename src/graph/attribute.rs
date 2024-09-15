use rustc_hash::FxHashMap;

pub struct InternalNodeAttribute;

impl InternalNodeAttribute {
    pub const EDGE_COUNT_ATTRIBUTE: &'static str = "$edges";
    pub const FROM_ATTRIBUTE: &'static str = "$from";
    pub const ID_ATTRIBUTE: &'static str = "$id";
    pub const NAME_ATTRIBUTE: &'static str = "$name";
    pub const TO_ATTRIBUTE: &'static str = "$to";
    pub const WEIGHT_ATTRIBUTE: &'static str = "$weight";

    pub fn get_identifier(attributes: &FxHashMap<String, String>) -> String {
        attributes.get(Self::ID_ATTRIBUTE).unwrap().clone()
    }
}