use sylvia::cw_schema::cw_serde;

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct AdminResponse {
    pub admin: Option<String>,
}

/// A group member has a weight associated with them.
/// This may all be equal, or may have meaning in the app that
/// makes use of the group (eg. voting power)
#[cw_serde(crate = "sylvia::cw_schema")]
pub struct Member {
    pub addr: String,
    pub weight: u64,
}

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct MemberListResponse {
    pub members: Vec<Member>,
}

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct MemberResponse {
    pub weight: Option<u64>,
}

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct TotalWeightResponse {
    pub weight: u64,
}

#[cw_serde(crate = "sylvia::cw_schema")]
pub struct HooksResponse {
    pub hooks: Vec<String>,
}
