use cosmwasm_std::{Addr, StdError};
use cw20_marketing::responses::{DownloadLogoResponse, LogoInfo, MarketingInfoResponse};
use cw20_marketing::{EmbeddedLogo, Logo};
use cw_multi_test::App;

use crate::contract::{InstantiateMarketingInfo, InstantiateMsgData};
use crate::error::ContractError;
use crate::multitest::proxy::Cw20BaseCodeId;

const PNG_HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

#[test]
fn update_unauthorised() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some("marketing".to_owned()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    let err = contract
        .update_marketing(
            &mut app,
            &owner,
            Some("New project".to_owned()),
            Some("Better description".to_owned()),
            Some("creator".to_owned()),
        )
        .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized {});

    // Ensure marketing didn't change
    let resp = contract.marketing_info(&app).unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(Addr::unchecked("marketing")),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.download_logo(&app).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found")
    );
}

#[test]
fn update_project() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(owner.to_string()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    contract
        .update_marketing(&mut app, &owner, Some("New project".to_owned()), None, None)
        .unwrap();

    let resp = contract.marketing_info(&app).unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("New project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(owner),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.download_logo(&app).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found")
    );
}

#[test]
fn clear_project() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(owner.to_string()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    contract
        .update_marketing(&mut app, &owner, Some("".to_owned()), None, None)
        .unwrap();

    let resp = contract.marketing_info(&app).unwrap();
    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: None,
            description: Some("Description".to_owned()),
            marketing: Some(owner),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.download_logo(&app).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found")
    );
}

#[test]
fn update_description() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(owner.to_string()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    contract
        .update_marketing(
            &mut app,
            &owner,
            None,
            Some("Better description".to_owned()),
            None,
        )
        .unwrap();

    let resp = contract.marketing_info(&app).unwrap();
    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Better description".to_owned()),
            marketing: Some(owner),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.download_logo(&app).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found")
    );
}

#[test]
fn clear_description() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(owner.to_string()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    contract
        .update_marketing(&mut app, &owner, None, Some("".to_owned()), None)
        .unwrap();

    let resp = contract.marketing_info(&app).unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: None,
            marketing: Some(owner),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.download_logo(&app).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found")
    );
}

#[test]
fn update_marketing() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(owner.to_string()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    contract
        .update_marketing(&mut app, &owner, None, None, Some("marketing".to_owned()))
        .unwrap();

    let resp = contract.marketing_info(&app).unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(Addr::unchecked("marketing")),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.download_logo(&app).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found")
    );
}

#[test]
fn update_marketing_invalid() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(owner.to_string()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    let err = contract
        .update_marketing(&mut app, &owner, None, None, Some("m".to_owned()))
        .unwrap_err();

    assert!(
        matches!(err, ContractError::Std(_)),
        "Expected Std error, received: {}",
        err
    );

    let resp = contract.marketing_info(&app).unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(owner),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.download_logo(&app).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found")
    );
}

#[test]
fn clear_marketing() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(owner.to_string()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    contract
        .update_marketing(&mut app, &owner, None, None, Some("".to_owned()))
        .unwrap();

    let resp = contract.marketing_info(&app).unwrap();
    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: None,
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.download_logo(&app).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found")
    );
}

#[test]
fn update_logo_url() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(owner.to_string()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    contract
        .upload_logo(&mut app, &owner, Logo::Url("new_url".to_owned()))
        .unwrap();

    let resp = contract.marketing_info(&app).unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(owner),
            logo: Some(LogoInfo::Url("new_url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.download_logo(&app).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found")
    );
}

#[test]
fn update_logo_png() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(owner.to_string()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    contract
        .upload_logo(
            &mut app,
            &owner,
            Logo::Embedded(EmbeddedLogo::Png(PNG_HEADER.into())),
        )
        .unwrap();

    let resp = contract.marketing_info(&app).unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(owner),
            logo: Some(LogoInfo::Embedded),
        }
    );

    let resp = contract.download_logo(&app).unwrap();
    assert_eq!(
        resp,
        DownloadLogoResponse {
            mime_type: "image/png".to_owned(),
            data: PNG_HEADER.into(),
        }
    );
}

#[test]
fn update_logo_svg() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let img = "<?xml version=\"1.0\"?><svg></svg>".as_bytes();

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(owner.to_string()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    contract
        .upload_logo(
            &mut app,
            &owner,
            Logo::Embedded(EmbeddedLogo::Svg(img.into())),
        )
        .unwrap();

    let resp = contract.marketing_info(&app).unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(owner),
            logo: Some(LogoInfo::Embedded),
        }
    );

    let resp = contract.download_logo(&app).unwrap();
    assert_eq!(
        resp,
        DownloadLogoResponse {
            mime_type: "image/svg+xml".to_owned(),
            data: img.into(),
        }
    );
}

#[test]
fn update_logo_png_oversized() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let img = [&PNG_HEADER[..], &[1; 6000][..]].concat();

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(owner.to_string()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    let err = contract
        .upload_logo(
            &mut app,
            &owner,
            Logo::Embedded(EmbeddedLogo::Png(img.into())),
        )
        .unwrap_err();

    assert_eq!(err, ContractError::LogoTooBig {});

    let resp = contract.marketing_info(&app).unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(owner),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.download_logo(&app).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found")
    );
}

#[test]
fn update_logo_svg_oversized() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let img = [
        "<?xml version=\"1.0\"?><svg>",
        std::str::from_utf8(&[b'x'; 6000]).unwrap(),
        "</svg>",
    ]
    .concat()
    .into_bytes();

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(owner.to_string()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    let err = contract
        .upload_logo(
            &mut app,
            &owner,
            Logo::Embedded(EmbeddedLogo::Svg(img.into())),
        )
        .unwrap_err();

    assert_eq!(err, ContractError::LogoTooBig {});

    let resp = contract.marketing_info(&app).unwrap();
    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(owner),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.download_logo(&app).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found")
    );
}

#[test]
fn update_logo_png_invalid() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let img = &[1];

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(owner.to_string()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    let err = contract
        .upload_logo(
            &mut app,
            &owner,
            Logo::Embedded(EmbeddedLogo::Png(img.into())),
        )
        .unwrap_err();
    assert_eq!(err, ContractError::InvalidPngHeader {});

    let resp = contract.marketing_info(&app).unwrap();
    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(owner),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.download_logo(&app).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found")
    );
}

#[test]
fn update_logo_svg_invalid() {
    let mut app = App::default();

    let owner = Addr::unchecked("addr0001");
    let img = &[1];

    let code_id = Cw20BaseCodeId::store_code(&mut app);

    let contract = code_id
        .instantiate(
            &mut app,
            &owner,
            InstantiateMsgData {
                name: "Cash Token".to_string(),
                symbol: "CASH".to_string(),
                decimals: 9,
                initial_balances: vec![],
                mint: None,
                marketing: Some(InstantiateMarketingInfo {
                    project: Some("Project".to_owned()),
                    description: Some("Description".to_owned()),
                    marketing: Some(owner.to_string()),
                    logo: Some(Logo::Url("url".to_owned())),
                }),
            },
            "Cw20 contract",
            None,
        )
        .unwrap();

    let err = contract
        .upload_logo(
            &mut app,
            &owner,
            Logo::Embedded(EmbeddedLogo::Svg(img.into())),
        )
        .unwrap_err();
    assert_eq!(err, ContractError::InvalidXmlPreamble {});

    let resp = contract.marketing_info(&app).unwrap();
    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(owner),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.download_logo(&app).unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found")
    );
}
