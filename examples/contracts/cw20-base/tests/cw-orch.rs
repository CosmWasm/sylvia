use cw20_base::contract::entry_points::*;
use cw20_base::contract::sv::*;
use cw20_base::contract::InstantiateMsgData;
use cw20_minting::responses::MinterResponse;
use cw20_minting::sv::Cw20MintingExecMsg;
use cw_orch::prelude::*;

#[cw_orch::interface(InstantiateMsg, ContractExecMsg, ContractQueryMsg, Empty)]
pub struct Cw20Base;

impl<Chain> Uploadable for Cw20Base<Chain> {
    /// Return the path to the wasm file corresponding to the contract
    fn wasm(_chain: &ChainInfoOwned) -> WasmPath {
        artifacts_dir_from_workspace!()
            .find_wasm_path("cw20_base")
            .unwrap()
    }
    /// Returns a CosmWasm contract wrapper
    fn wrapper() -> Box<dyn MockContract<Empty>> {
        Box::new(ContractWrapper::new_with_empty(execute, instantiate, query))
    }
}
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
                initial_balances: vec![],
                marketing: None,
            },
        },
        None,
        None,
    )?;
    contract.mint(150_000u128.into(), mock.sender().to_string())?;
    let balance = contract.balance(mock.sender().to_string())?;

    assert_eq!(balance.balance.u128(), 150_000u128);

    Ok(())
}
