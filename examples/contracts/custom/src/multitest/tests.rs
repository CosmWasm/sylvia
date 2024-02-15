use sylvia::multitest::App;

use crate::contract::sv::multitest_utils::CodeId;

use super::custom_module::CustomModule;

#[test]
fn test_custom() {
    let owner = "owner";

    let mt_app = cw_multi_test::BasicAppBuilder::new_custom()
        .with_custom(CustomModule::default())
        .build(|router, _, storage| {
            router.custom.init_counter(storage).unwrap();
        });

    let app = App::new(mt_app);

    let code_id = CodeId::store_code(&app);

    let contract = code_id.instantiate().call(owner).unwrap();

    contract.send_custom().call(owner).unwrap();

    let count = contract.query_exec().unwrap().count;
    assert_eq!(count, 1);
    let count = contract.query_sudo().unwrap().count;
    assert_eq!(count, 0);

    contract.sudo_custom().unwrap();

    let count = contract.query_exec().unwrap().count;
    assert_eq!(count, 1);
    let count = contract.query_sudo().unwrap().count;
    assert_eq!(count, 1);
}
