use sylvia::multitest::App;

use crate::contract::sv::mt::CodeId;

use super::custom_module::CustomModule;

use crate::contract::sv::mt::CustomContractProxy;

use cosmwasm_std::CosmosMsg;
use cw1::sv::mt::Cw1Proxy;

#[test]
fn test_custom() {
    let owner = "owner";

    let mt_app = cw_multi_test::BasicAppBuilder::new_custom()
        .with_custom(CustomModule::default())
        .build(|router, _, storage| {
            router.custom.save_counter(storage, 0).unwrap();
        });

    let app = App::new(mt_app);

    let code_id = CodeId::store_code(&app);

    let contract = code_id.instantiate().call(owner).unwrap();

    contract.send_custom().call(owner).unwrap();

    contract
        .can_execute("".to_string(), CosmosMsg::Custom(cosmwasm_std::Empty {}))
        .unwrap();
    contract.execute(vec![]).call(owner).unwrap();

    let count = contract.query_custom().unwrap().count;
    assert_eq!(count, 1);

    contract.increment_sudo_counter().unwrap();

    let count = contract.sudo_counter().unwrap().count;
    assert_eq!(count, 1);
}
