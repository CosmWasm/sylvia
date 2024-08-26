use sylvia::cw_multi_test::{BasicAppBuilder, IntoBech32};
use sylvia::multitest::App;

use super::custom_module::CustomModule;
use crate::contract::sv::mt::CodeId;
use crate::contract::sv::mt::CustomContractProxy;

use cw1::sv::mt::Cw1Proxy;
use sylvia::cw_std::CosmosMsg;

#[test]
fn test_custom() {
    let owner = "owner".into_bech32();

    let mt_app = BasicAppBuilder::new_custom()
        .with_custom(CustomModule::default())
        .build(|router, _, storage| {
            router.custom.save_counter(storage, 0).unwrap();
        });

    let app = App::new(mt_app);

    let code_id = CodeId::store_code(&app);

    let contract = code_id.instantiate().call(&owner).unwrap();

    contract.send_custom().call(&owner).unwrap();

    contract
        .can_execute("".to_string(), CosmosMsg::Custom(sylvia::cw_std::Empty {}))
        .unwrap();
    contract.execute(vec![]).call(&owner).unwrap();

    let count = contract.query_custom().unwrap().count;
    assert_eq!(count, 1);

    contract.increment_sudo_counter().unwrap();

    let count = contract.sudo_counter().unwrap().count;
    assert_eq!(count, 1);
}
