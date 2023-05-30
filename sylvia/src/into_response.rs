use cosmwasm_std::{CosmosMsg, Empty, Response, StdError, StdResult, SubMsg};

/// Trait converting `SubMsg` to one carrying another chain-custom message
pub trait IntoMsg<C> {
    fn into_msg(self) -> StdResult<SubMsg<C>>;
}

/// `SubMsg<Empty>` can be made into any `SubMsg<C>`
impl<C> IntoMsg<C> for SubMsg<Empty> {
    fn into_msg(self) -> StdResult<SubMsg<C>> {
        let msg = match self.msg {
            CosmosMsg::Wasm(wasm) => CosmosMsg::Wasm(wasm),
            CosmosMsg::Bank(bank) => CosmosMsg::Bank(bank),
            #[cfg(feature = "staking")]
            CosmosMsg::Staking(staking) => CosmosMsg::Staking(staking),
            #[cfg(feature = "staking")]
            CosmosMsg::Distribution(distribution) => CosmosMsg::Distribution(distribution),
            CosmosMsg::Custom(_) => Err(StdError::generic_err(
                "Custom Empty message should not be sent",
            ))?,
            #[cfg(feature = "stargate")]
            CosmosMsg::Ibc(ibc) => CosmosMsg::Ibc(ibc),
            #[cfg(feature = "stargate")]
            CosmosMsg::Stargate { type_url, value } => CosmosMsg::Stargate { type_url, value },
            #[cfg(feature = "stargate")]
            CosmosMsg::Gov(msg) => CosmosMsg::Gov(msg),
            _ => Err(StdError::generic_err(format!(
                "Unknown message variant: {:?}. Please make sure you are using up-to-date Sylvia version, and if so please issue this bug on the Sylvia repository.",
                self
            )))?,
        };

        Ok(SubMsg {
            msg,
            id: self.id,
            gas_limit: self.gas_limit,
            reply_on: self.reply_on,
        })
    }
}

pub trait IntoResponse<T> {
    fn into_response(self) -> StdResult<Response<T>>;
}

impl<T> IntoResponse<T> for Response<Empty> {
    fn into_response(self) -> StdResult<Response<T>> {
        let messages = self
            .messages
            .into_iter()
            .map(|msg| msg.into_msg())
            .collect::<StdResult<Vec<_>>>()?;
        let mut resp = Response::new()
            .add_submessages(messages.into_iter())
            .add_events(self.events)
            .add_attributes(self.attributes);
        resp.data = self.data;

        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{CosmosMsg, CustomMsg, Empty, Response, StdError};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use super::IntoResponse;

    #[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
    struct MyMsg {}

    impl CustomMsg for MyMsg {}

    #[test]
    fn into_custom() {
        let resp = Response::<Empty>::default();

        let _: Response<MyMsg> = resp.into_response().unwrap();
    }

    #[test]
    fn empty_custom_msg() {
        let mut resp = Response::<Empty>::default();
        resp = resp.add_message(CosmosMsg::Custom(Empty {}));

        let err = IntoResponse::<MyMsg>::into_response(resp).unwrap_err();
        assert_eq!(
            err,
            StdError::generic_err("Custom Empty message should not be sent")
        );
    }
}
