use std::cell::RefCell;

use cosmwasm_std::{Addr, Coin};

#[derive(Default)]
pub struct App {
    pub app: RefCell<cw_multi_test::App>,
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct ExecParams<'a> {
    pub sender: &'a Addr,
    pub funds: &'a [Coin],
}

impl<'a> ExecParams<'a> {
    pub fn new(sender: &'a Addr, funds: &'a [Coin]) -> Self {
        Self { sender, funds }
    }
}

#[derive(Clone, PartialEq, Debug, Eq)]
pub struct InstantiateParams<'a> {
    pub sender: &'a Addr,
    pub funds: &'a [Coin],
    pub label: &'a str,
    pub admin: Option<String>,
}

impl<'a> InstantiateParams<'a> {
    pub fn new(sender: &'a Addr, funds: &'a [Coin], label: &'a str, admin: Option<String>) -> Self {
        Self {
            sender,
            funds,
            label,
            admin,
        }
    }
}
