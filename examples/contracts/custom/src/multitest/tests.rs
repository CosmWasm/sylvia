use sylvia::multitest::App;

use crate::contract::multitest_utils::CodeId;

use super::custom_module::CustomModule;

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

    let count = contract.query_custom().unwrap().count;
    assert_eq!(count, 1);
}
