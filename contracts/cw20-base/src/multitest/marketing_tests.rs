use cosmwasm_std::{Addr, StdError};
use cw20_marketing::responses::{DownloadLogoResponse, LogoInfo, MarketingInfoResponse};
use cw20_marketing::{EmbeddedLogo, Logo};
use sylvia::multitest::App;

use crate::contract::multitest_utils::CodeId;
use crate::contract::{InstantiateMarketingInfo, InstantiateMsgData};
use crate::error::ContractError;
use crate::marketing::test_utils::Cw20MarketingMethods;

const PNG_HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

#[test]
fn update_unauthorised() {
    let app = App::default();

    let owner = "addr0001";

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
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
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    let err = contract
        .cw20_marketing_proxy()
        .update_marketing(
            Some("New project".to_owned()),
            Some("Better description".to_owned()),
            Some("creator".to_owned()),
        )
        .call(owner)
        .unwrap_err();

    assert_eq!(err, ContractError::Unauthorized {});

    // Ensure marketing didn't change
    let resp = contract.cw20_marketing_proxy().marketing_info().unwrap();

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
    let err = contract.cw20_marketing_proxy().download_logo().unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found").into()
    );
}

#[test]
fn update_project() {
    let app = App::default();

    let owner = "addr0001";

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
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
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    contract
        .cw20_marketing_proxy()
        .update_marketing(Some("New project".to_owned()), None, None)
        .call(owner)
        .unwrap();

    let resp = contract.cw20_marketing_proxy().marketing_info().unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("New project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(Addr::unchecked(owner)),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.cw20_marketing_proxy().download_logo().unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found").into()
    );
}

#[test]
fn clear_project() {
    let app = App::default();

    let owner = "addr0001";

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
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
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    contract
        .cw20_marketing_proxy()
        .update_marketing(Some("".to_owned()), None, None)
        .call(owner)
        .unwrap();

    let resp = contract.cw20_marketing_proxy().marketing_info().unwrap();
    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: None,
            description: Some("Description".to_owned()),
            marketing: Some(Addr::unchecked(owner)),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.cw20_marketing_proxy().download_logo().unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found").into()
    );
}

#[test]
fn update_description() {
    let app = App::default();

    let owner = "addr0001";

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
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
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    contract
        .cw20_marketing_proxy()
        .update_marketing(None, Some("Better description".to_owned()), None)
        .call(owner)
        .unwrap();

    let resp = contract.cw20_marketing_proxy().marketing_info().unwrap();
    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Better description".to_owned()),
            marketing: Some(Addr::unchecked(owner)),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.cw20_marketing_proxy().download_logo().unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found").into()
    );
}

#[test]
fn clear_description() {
    let app = App::default();

    let owner = "addr0001";

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
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
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    contract
        .cw20_marketing_proxy()
        .update_marketing(None, Some("".to_owned()), None)
        .call(owner)
        .unwrap();

    let resp = contract.cw20_marketing_proxy().marketing_info().unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: None,
            marketing: Some(Addr::unchecked(owner)),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.cw20_marketing_proxy().download_logo().unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found").into()
    );
}

#[test]
fn update_marketing() {
    let app = App::default();

    let owner = "addr0001";

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
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
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    contract
        .cw20_marketing_proxy()
        .update_marketing(None, None, Some("marketing".to_owned()))
        .call(owner)
        .unwrap();

    let resp = contract.cw20_marketing_proxy().marketing_info().unwrap();

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
    let err = contract.cw20_marketing_proxy().download_logo().unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found").into()
    );
}

#[test]
fn update_marketing_invalid() {
    let app = App::default();

    let owner = "addr0001";

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
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
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    let err = contract
        .cw20_marketing_proxy()
        .update_marketing(None, None, Some("m".to_owned()))
        .call(owner)
        .unwrap_err();

    assert!(
        matches!(err, ContractError::Std(_)),
        "Expected Std error, received: {}",
        err
    );

    let resp = contract.cw20_marketing_proxy().marketing_info().unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(Addr::unchecked(owner)),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.cw20_marketing_proxy().download_logo().unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found").into()
    );
}

#[test]
fn clear_marketing() {
    let app = App::default();

    let owner = "addr0001";

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
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
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    contract
        .cw20_marketing_proxy()
        .update_marketing(None, None, Some("".to_owned()))
        .call(owner)
        .unwrap();

    let resp = contract.cw20_marketing_proxy().marketing_info().unwrap();
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
    let err = contract.cw20_marketing_proxy().download_logo().unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found").into()
    );
}

#[test]
fn update_logo_url() {
    let app = App::default();

    let owner = "addr0001";

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
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
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    contract
        .cw20_marketing_proxy()
        .upload_logo(Logo::Url("new_url".to_owned()))
        .call(owner)
        .unwrap();

    let resp = contract.cw20_marketing_proxy().marketing_info().unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(Addr::unchecked(owner)),
            logo: Some(LogoInfo::Url("new_url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.cw20_marketing_proxy().download_logo().unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found").into()
    );
}

#[test]
fn update_logo_png() {
    let app = App::default();

    let owner = "addr0001";

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
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
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    contract
        .cw20_marketing_proxy()
        .upload_logo(Logo::Embedded(EmbeddedLogo::Png(PNG_HEADER.into())))
        .call(owner)
        .unwrap();

    let resp = contract.cw20_marketing_proxy().marketing_info().unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(Addr::unchecked(owner)),
            logo: Some(LogoInfo::Embedded),
        }
    );

    let resp = contract.cw20_marketing_proxy().download_logo().unwrap();
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
    let app = App::default();

    let owner = "addr0001";
    let img = "<?xml version=\"1.0\"?><svg></svg>".as_bytes();

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
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
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    contract
        .cw20_marketing_proxy()
        .upload_logo(Logo::Embedded(EmbeddedLogo::Svg(img.into())))
        .call(owner)
        .unwrap();

    let resp = contract.cw20_marketing_proxy().marketing_info().unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(Addr::unchecked(owner)),
            logo: Some(LogoInfo::Embedded),
        }
    );

    let resp = contract.cw20_marketing_proxy().download_logo().unwrap();
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
    let app = App::default();

    let owner = "addr0001";
    let img = [&PNG_HEADER[..], &[1; 6000][..]].concat();

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
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
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    let err = contract
        .cw20_marketing_proxy()
        .upload_logo(Logo::Embedded(EmbeddedLogo::Png(img.into())))
        .call(owner)
        .unwrap_err();

    assert_eq!(err, ContractError::LogoTooBig {});

    let resp = contract.cw20_marketing_proxy().marketing_info().unwrap();

    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(Addr::unchecked(owner)),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.cw20_marketing_proxy().download_logo().unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found").into()
    );
}

#[test]
fn update_logo_svg_oversized() {
    let app = App::default();

    let owner = "addr0001";
    let img = [
        "<?xml version=\"1.0\"?><svg>",
        std::str::from_utf8(&[b'x'; 6000]).unwrap(),
        "</svg>",
    ]
    .concat()
    .into_bytes();

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
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
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    let err = contract
        .cw20_marketing_proxy()
        .upload_logo(Logo::Embedded(EmbeddedLogo::Svg(img.into())))
        .call(owner)
        .unwrap_err();

    assert_eq!(err, ContractError::LogoTooBig {});

    let resp = contract.cw20_marketing_proxy().marketing_info().unwrap();
    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(Addr::unchecked(owner)),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.cw20_marketing_proxy().download_logo().unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found").into()
    );
}

#[test]
fn update_logo_png_invalid() {
    let app = App::default();

    let owner = "addr0001";
    let img = &[1];

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
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
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    let err = contract
        .cw20_marketing_proxy()
        .upload_logo(Logo::Embedded(EmbeddedLogo::Png(img.into())))
        .call(owner)
        .unwrap_err();
    assert_eq!(err, ContractError::InvalidPngHeader {});

    let resp = contract.cw20_marketing_proxy().marketing_info().unwrap();
    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(Addr::unchecked(owner)),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.cw20_marketing_proxy().download_logo().unwrap_err();
    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found").into()
    );
}

#[test]
fn update_logo_svg_invalid() {
    let app = App::default();

    let owner = "addr0001";
    let img = &[1];

    let code_id = CodeId::store_code(&app);

    let contract = code_id
        .instantiate(InstantiateMsgData {
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
        })
        .with_label("Cw20 contract")
        .call(owner)
        .unwrap();

    let err = contract
        .cw20_marketing_proxy()
        .upload_logo(Logo::Embedded(EmbeddedLogo::Svg(img.into())))
        .call(owner)
        .unwrap_err();
    assert_eq!(err, ContractError::InvalidXmlPreamble {});

    let resp = contract.cw20_marketing_proxy().marketing_info().unwrap();
    assert_eq!(
        resp,
        MarketingInfoResponse {
            project: Some("Project".to_owned()),
            description: Some("Description".to_owned()),
            marketing: Some(Addr::unchecked(owner)),
            logo: Some(LogoInfo::Url("url".to_owned())),
        }
    );

    // Due to QuerierWrapper impl it will return generic error instead of forwarding ContractError
    let err = contract.cw20_marketing_proxy().download_logo().unwrap_err();

    assert_eq!(
        err,
        StdError::generic_err("Querier contract error: logo not found").into()
    );
}
