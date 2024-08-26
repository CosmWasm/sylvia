use crate::state::Permissions;
use cw_utils::{Expiration, NativeBalance};
use sylvia::cw_schema::schemars::JsonSchema;
use sylvia::cw_std::Addr;
use sylvia::schemars;
use sylvia::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug, Default)]
#[serde(crate = "sylvia::serde")]
pub struct AllAllowancesResponse {
    pub allowances: Vec<AllowanceInfo>,
}

#[cfg(test)]
impl AllAllowancesResponse {
    pub fn canonical(mut self) -> Self {
        self.allowances = self
            .allowances
            .into_iter()
            .map(AllowanceInfo::canonical)
            .collect();
        self.allowances.sort_by(AllowanceInfo::cmp_by_spender);
        self
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(crate = "sylvia::serde")]
pub struct AllowanceInfo {
    pub spender: Addr,
    pub balance: NativeBalance,
    pub expires: Expiration,
}

#[cfg(test)]
impl AllowanceInfo {
    /// Utility function providing some ordering to be used with `slice::sort_by`.
    ///
    /// Note, that this doesn't implement full ordering - items with same spender but differing on
    /// permissions, would be considered equal, however as spender is a unique key in any valid
    /// state this is enough for testing purposes.
    ///
    /// Example:
    ///
    /// ```
    /// # use cw_utils::{Expiration, NativeBalance};
    /// # use cw1_subkeys::msg::AllowanceInfo;
    /// # use sylvia::cw_schema::{cw_serde, QueryResponses};
    /// # use sylvia::cw_std::coin;
    ///
    /// let mut allows = vec![Allowance {
    ///   spender: "spender2".to_owned(),
    ///   balance: NativeBalance(vec![coin(1, "token1")]),
    ///   expires: Expiration::Never {},
    /// }, Allowance {
    ///   spender: "spender1".to_owned(),
    ///   balance: NativeBalance(vec![coin(2, "token2")]),
    ///   expires: Expiration::Never {},
    /// }];
    ///
    /// allows.sort_by(Allowance::cmp_by_spender);
    ///
    /// assert_eq!(
    ///   allows.into_iter().map(|allow| allow.spender).collect::<Vec<_>>(),
    ///   vec!["spender1".to_owned(), "spender2".to_owned()]
    /// );
    /// ```
    pub fn cmp_by_spender(left: &Self, right: &Self) -> std::cmp::Ordering {
        left.spender.cmp(&right.spender)
    }

    pub fn canonical(mut self) -> Self {
        self.balance.normalize();
        self
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug, Default)]
#[serde(crate = "sylvia::serde")]
pub struct AllPermissionsResponse {
    pub permissions: Vec<PermissionsInfo>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema, Debug)]
#[serde(crate = "sylvia::serde")]
pub struct PermissionsInfo {
    pub spender: Addr,
    pub permissions: Permissions,
}

#[cfg(any(test, feature = "mt"))]
impl PermissionsInfo {
    /// Utility function providing some ordering to be used with `slice::sort_by`.
    ///
    /// Note, that this doesn't implement full ordering - items with same spender but differing on
    /// permissions, would be considered equal, however as spender is a unique key in any valid
    /// state this is enough for testing purposes.
    ///
    /// Example:
    ///
    /// ```
    /// # use cw1_subkeys::msg::PermissionsInfo;
    /// # use cw1_subkeys::state::Permissions;
    ///
    /// let mut perms = vec![PermissionsInfo {
    ///   spender: "spender2".to_owned(),
    ///   permissions: Permissions::default(),
    /// }, PermissionsInfo {
    ///   spender: "spender1".to_owned(),
    ///   permissions: Permissions::default(),
    /// }];
    ///
    /// perms.sort_by(PermissionsInfo::cmp_by_spender);
    ///
    /// assert_eq!(
    ///   perms.into_iter().map(|perm| perm.spender).collect::<Vec<_>>(),
    ///   vec!["spender1".to_owned(), "spender2".to_owned()]
    /// );
    /// ```
    pub fn cmp_by_spender(left: &Self, right: &Self) -> std::cmp::Ordering {
        left.spender.cmp(&right.spender)
    }

    pub fn spender(&self) -> &str {
        self.spender.as_ref()
    }
}
