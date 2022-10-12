use cosmwasm_std::from_binary;
use cosmwasm_std::{coins, Addr, Decimal};
use cw_multi_test::{App, AppResponse, Executor};

use super::admin_vote_contracts::admin;
use super::admin_vote_contracts::ProposeAdminResp;
#[allow(unused_imports)]
use super::admin_vote_contracts::{AdminContract, VoteContract};

#[test]
fn propose_admin() {
    let mut app = App::new(|router, _api, storage| {
        router
            .bank
            .init_balance(storage, &Addr::unchecked("admin1"), coins(100, "utgd"))
            .unwrap();
    });
    let admin_code_id = app.store_code(Box::new(AdminContract {}));
    let vote_code_id = app.store_code(Box::new(VoteContract {}));

    let admin = app
        .instantiate_contract(
            admin_code_id,
            Addr::unchecked("owner"),
            &admin::InstantiateMsg {
                admins: vec![
                    String::from("owner"),
                    String::from("admin1"),
                    String::from("admin2"),
                    String::from("admin3"),
                ],
                vote_code_id,
                quorum: Decimal::percent(75),
            },
            &[],
            "admin",
            None,
        )
        .unwrap();

    let resp: AppResponse = app
        .execute_contract(
            Addr::unchecked("owner"),
            admin,
            &admin::ExecMsg::ProposeAdmin {
                addr: String::from("new_admin"),
                admin_code_id,
            },
            &[],
        )
        .unwrap();

    let _: ProposeAdminResp = from_binary(&resp.data.unwrap()).unwrap();
}
