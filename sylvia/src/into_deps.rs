use cosmwasm_std::{CustomQuery, Deps, DepsMut, Empty, QuerierWrapper};

/// Trait converting `Deps` to one operating on another `Query` type. By default only conversions
/// from any `Deps<Q>` to `Deps<Empty>` are possible, and in general - only converting to `Deps`
/// over simpler query (being a subset of the original one) should be allowed.
trait IntoDeps<'deps, Q>
where
    Q: CustomQuery,
{
    fn into_deps(self) -> Deps<'deps, Q>;
}

/// Any `Deps<Q>` can be made into `Deps<Empty>`
///
/// It would be better to define it on owned `Deps`, but the `QuerierWrapper::querier` is not
/// accessible - some destructuring function for it would be helpfull here
impl<'deps, Q> IntoDeps<'deps, Empty> for &'deps Deps<'deps, Q>
where
    Q: CustomQuery,
{
    fn into_deps(self) -> Deps<'deps, Empty> {
        Deps {
            storage: self.storage,
            api: self.api,
            querier: QuerierWrapper::new(&*self.querier),
        }
    }
}

/// Trait converting `DepsMut` to one operating on another `Query` type. By default only
/// conversions from any `DepsMut<Q>` to `DepsMut<Empty>` are possible, and in general - only
/// converting to `DepsMut` over simpler query (being a subset of the original one) should be
/// allowed.
trait IntoDepsMut<'deps, Q>
where
    Q: CustomQuery,
{
    fn into_deps_mut(self) -> DepsMut<'deps, Q>;
}

/// Any `DepsMut<Q>` can be made into `DepsMut<Empty>`
///
/// It would be better to define it on owned `DepsMut`, but the `QuerierWrapper::querier` is not
/// accessible - some destructuring function for it would be helpfull here
impl<'deps, Q> IntoDepsMut<'deps, Empty> for &'deps mut DepsMut<'deps, Q>
where
    Q: CustomQuery,
{
    fn into_deps_mut(self) -> DepsMut<'deps, Empty> {
        DepsMut {
            storage: self.storage,
            api: self.api,
            querier: QuerierWrapper::new(&*self.querier),
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, MockApi, MockStorage};
    use cosmwasm_std::{CustomQuery, Deps, DepsMut, Empty, QuerierWrapper};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use crate::into_deps::IntoDeps;

    use super::IntoDepsMut;

    #[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, PartialEq, Eq)]
    struct MyQuery {}

    impl CustomQuery for MyQuery {}

    #[test]
    fn empty_into_deps() {
        let deps = mock_dependencies();
        let storage = MockStorage::new();
        let api = MockApi::default();
        let querier = QuerierWrapper::<Empty>::new(&deps.querier);

        let deps = Deps {
            storage: &storage,
            api: &api,
            querier,
        };
        let _: Deps<Empty> = deps.into_deps();
    }

    #[test]
    fn custom_into_deps() {
        let deps = mock_dependencies();
        let storage = MockStorage::new();
        let api = MockApi::default();
        let querier = QuerierWrapper::<MyQuery>::new(&deps.querier);

        let deps = Deps {
            storage: &storage,
            api: &api,
            querier,
        };
        let _: Deps<Empty> = deps.into_deps();
    }

    #[test]
    fn empty_into_deps_mut() {
        let deps = mock_dependencies();
        let mut storage = MockStorage::new();
        let api = MockApi::default();
        let querier = QuerierWrapper::<Empty>::new(&deps.querier);

        let mut deps = DepsMut {
            storage: &mut storage,
            api: &api,
            querier,
        };
        let _: DepsMut<Empty> = deps.into_deps_mut();
    }

    #[test]
    fn custom_into_deps_mut() {
        let deps = mock_dependencies();
        let mut storage = MockStorage::new();
        let api = MockApi::default();
        let querier = QuerierWrapper::<MyQuery>::new(&deps.querier);

        let mut deps = DepsMut {
            storage: &mut storage,
            api: &api,
            querier,
        };
        let _: DepsMut<Empty> = deps.into_deps_mut();
    }
}
