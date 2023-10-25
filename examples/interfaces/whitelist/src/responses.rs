use serde::{Deserialize, Serialize};
use sylvia::schemars;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, schemars::JsonSchema, Debug, Default)]
pub struct AdminListResponse {
    pub admins: Vec<String>,
    pub mutable: bool,
}

#[cfg(any(test, feature = "test-utils"))]
impl AdminListResponse {
    /// Utility function forconverting message to its canonical form, so two messages with
    /// different representation but same semantical meaning can be easly compared.
    ///
    /// It could be encapsulated in custom `PartialEq` implementation, but `PartialEq` is expected
    /// to be quickly, so it seems to be reasonable to keep it as representation-equality, and
    /// canonicalize message only when it is needed
    ///
    /// Example:
    ///
    /// ```
    /// # use cw1_whitelist::msg::AdminListResponse;
    ///
    /// let resp1 = AdminListResponse {
    ///   admins: vec!["admin1".to_owned(), "admin2".to_owned()],
    ///   mutable: true,
    /// };
    ///
    /// let resp2 = AdminListResponse {
    ///   admins: vec!["admin2".to_owned(), "admin1".to_owned(), "admin2".to_owned()],
    ///   mutable: true,
    /// };
    ///
    /// assert_eq!(resp1.canonical(), resp2.canonical());
    /// ```
    pub fn canonical(mut self) -> Self {
        self.admins.sort();
        self.admins.dedup();
        self
    }
}
