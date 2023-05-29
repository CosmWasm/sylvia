use cosmwasm_std::{CosmosMsg, Empty, Response, SubMsg};

/// Trait converting `SubMsg` to one carrying another chain-custom message
pub trait IntoMsg<C> {
    fn into_msg(self) -> SubMsg<C>;
}

/// `SubMsg<Empty>` can be made into any `SubMsg<C>`
impl<C> IntoMsg<C> for SubMsg<Empty> {
    fn into_msg(self) -> SubMsg<C> {
        SubMsg {
            msg: match self.msg {
                CosmosMsg::Wasm(wasm) => CosmosMsg::Wasm(wasm),
                CosmosMsg::Bank(bank) => CosmosMsg::Bank(bank),
                CosmosMsg::Staking(staking) => CosmosMsg::Staking(staking),
                CosmosMsg::Distribution(distribution) => CosmosMsg::Distribution(distribution),
                CosmosMsg::Custom(_) => unreachable!(),
                CosmosMsg::Ibc(ibc) => CosmosMsg::Ibc(ibc),
                CosmosMsg::Stargate { type_url, value } => CosmosMsg::Stargate { type_url, value },
                _ => panic!("unknown message variant {:?}", self),
            },
            id: self.id,
            gas_limit: self.gas_limit,
            reply_on: self.reply_on,
        }
    }
}

pub trait IntoResponse<T> {
    fn into_response(self) -> Response<T>;
}

impl<T> IntoResponse<T> for Response<Empty> {
    fn into_response(self) -> Response<T> {
        let mut resp = Response::new()
            .add_submessages(self.messages.into_iter().map(IntoMsg::into_msg))
            .add_events(self.events)
            .add_attributes(self.attributes);
        resp.data = self.data;

        resp
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::{CustomMsg, Empty, Response};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use super::IntoResponse;

    #[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
    struct MyMsg {}

    impl CustomMsg for MyMsg {}

    #[test]
    fn into_custom() {
        let resp = Response::<Empty>::default();

        let _: Response<MyMsg> = resp.into_response();
    }
}
