use cw20_base::contract::sv::*;
use cw20_base::contract::InstantiateMsgData;
use cw20_base::orch::Cw20Base;
use cw20_base::responses::Cw20Coin;
use cw20_minting::responses::MinterResponse;
use cw20_minting::sv::Cw20MintingExecMsgFns;
use cw_orch::prelude::*;

#[test]
fn mock_interact() -> cw_orch::anyhow::Result<()> {
    let mock = MockBech32::new("mock");

    let contract = Cw20Base::new("cw20base", mock.clone());

    contract.upload()?;

    contract.instantiate(
        &InstantiateMsg {
            data: InstantiateMsgData {
                name: "cw20-test".to_string(),
                symbol: "CWORCH".to_string(),
                decimals: 6,
                mint: Some(MinterResponse {
                    minter: mock.sender().to_string(),
                    cap: None,
                }),
                initial_balances: vec![Cw20Coin {
                    address: mock.sender().to_string(),
                    amount: 150_000u128.into(),
                }],
                marketing: None,
            },
        },
        None,
        None,
    )?;

    let balance = contract.balance(mock.sender().to_string())?;
    assert_eq!(balance.balance.u128(), 150_000u128);

    // TODO: how do i get a different address for testing???
    // Here I just send to self
    let friend = MockBech32::new("mock");
    contract.transfer(10_000u128.into(), friend.sender().to_string())?;
    assert_eq!(friend.sender().to_string(), mock.sender().to_string());
    let balance = contract.balance(mock.sender().to_string())?;
    assert_eq!(balance.balance.u128(), 150_000u128);

    // This is what fails... let's see how to make that happen
    contract.mint(150_000u128.into(), mock.sender().to_string())?;
    // contract.execute(
    //     &ContractExecMsg::Minting(Cw20MintingExecMsg::Mint {
    //         recipient: mock.sender().to_string(),
    //         amount: 150_000u128.into(),
    //     }),
    //     None,
    // )?;
    let balance = contract.balance(mock.sender().to_string())?;
    assert_eq!(balance.balance.u128(), 300_000u128);

    Ok(())
}
