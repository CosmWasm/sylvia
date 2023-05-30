use cosmwasm_std::{CustomQuery, Deps, DepsMut, Empty, QuerierWrapper};

trait IntoDeps<'deps, Q>
where
    Q: CustomQuery,
{
    fn into_deps(self) -> Deps<'deps, Q>;
}

trait IntoDepsMut<'deps, Q>
where
    Q: CustomQuery,
{
    fn into_deps_mut(self) -> DepsMut<'deps, Q>;
}

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
