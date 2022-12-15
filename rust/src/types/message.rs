use super::Calldata;
use crate::types::Address;
use ruint::aliases::U256;

#[derive(Debug)]
/// Items that are used by contract creation or message call.
pub enum Message<'a, 'b>
where
    'a: 'b,
{
    Create {
        caller: &'a Address,
        gas: &'a U256,
        value: &'a U256,
        data: &'b Calldata<'a>,
    },
    Call {
        caller: &'a Address,
        target: &'a Address,
        gas: &'a U256,
        value: &'a U256,
        data: &'b Calldata<'a>,
    },
    Staticcall {
        caller: &'a Address,
        target: &'a Address,
        gas: &'a U256,
        value: &'a U256,
        data: &'b Calldata<'a>,
    },
}

impl<'a, 'b> Message<'a, 'b>
where
    'a: 'b,
{
    pub(crate) fn new(
        caller: &'a Address,
        target: &'a Option<Address>,
        gas: &'a U256,
        value: &'a U256,
        data: &'b Calldata<'a>,
    ) -> Self {
        if let Some(target) = target {
            Self::Call {
                caller,
                target,
                gas,
                value,
                data,
            }
        } else {
            Self::Create {
                caller,
                gas,
                value,
                data,
            }
        }
    }

    pub(crate) fn staticcall(
        caller: &'a Address,
        target: &'a Address,
        gas: &'a U256,
        value: &'a U256,
        data: &'b Calldata<'a>,
    ) -> Self {
        Self::Staticcall {
            caller,
            target,
            gas,
            value,
            data,
        }
    }

    pub(crate) fn caller(&self) -> &Address {
        use Message::*;
        match self {
            Call { caller, .. } | Staticcall { caller, .. } | Create { caller, .. } => caller,
        }
    }

    pub(crate) fn target(&self) -> &Address {
        use Message::*;
        match self {
            Call { target, .. } | Staticcall { target, .. } => target,
            Create { .. } => todo!(),
        }
    }

    pub(crate) fn value(&self) -> &U256 {
        use Message::*;
        match self {
            Call { value, .. } | Staticcall { value, .. } | Create { value, .. } => &value,
        }
    }

    pub(crate) fn data(&self) -> &Calldata {
        use Message::*;
        match &self {
            Call { data, .. } | Staticcall { data, .. } | Create { data, .. } => &data,
        }
    }

    pub(crate) fn is_staticcall(&self) -> bool {
        use Message::*;
        match self {
            Staticcall { .. } => true,
            _ => false,
        }
    }
}
