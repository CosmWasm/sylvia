use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct AdminResponse {
    pub admin: Option<String>,
}

/// A group member has a weight associated with them.
/// This may all be equal, or may have meaning in the app that
/// makes use of the group (eg. voting power)
#[cw_serde]
pub struct Member {
    pub addr: String,
    pub weight: u64,
}

#[cw_serde]
pub struct MemberListResponse {
    pub members: Vec<Member>,
}

#[cw_serde]
pub struct MemberResponse {
    pub weight: Option<u64>,
}

#[cw_serde]
pub struct TotalWeightResponse {
    pub weight: u64,
}

#[cw_serde]
pub struct HooksResponse {
    pub hooks: Vec<String>,
}
