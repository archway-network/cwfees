use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, Coin, CosmosMsg};
use prost::DecodeError;
use prost::Message;

pub mod archway {
    pub mod cwfees {
        pub mod v1 {
            include!("archway.cwfees.v1.rs");
        }
    }
}

/// The function will return a CosmosMsg which contains a stargate
/// message that will register the provided contract address as a
/// fee granter.
pub fn new_register_as_granter_msg(contract_address: impl Into<String>) -> CosmosMsg {
    const TYPE_URL: &str = "/archway.cwfees.v1.MsgRegisterAsGranter";
    return CosmosMsg::Stargate {
        type_url: TYPE_URL.to_string(),
        value: Binary::from(
            archway::cwfees::v1::MsgRegisterAsGranter {
                granting_contract: contract_address.into(),
            }
            .encode_to_vec(),
        ),
    };
}

/// The function will return a CosmosMsg which contains a stargate message
/// that will unregister the provided contract address as a fee granter.
pub fn new_unregister_as_granter_msg(contract_address: impl Into<String>) -> CosmosMsg {
    const TYPE_URL: &str = "/archway.cwfees.v1.MsgUnregisterAsGranter";
    return CosmosMsg::Stargate {
        type_url: TYPE_URL.to_string(),
        value: Binary::from(
            archway::cwfees::v1::MsgUnregisterAsGranter {
                granting_contract: contract_address.into(),
            }
            .encode_to_vec(),
        ),
    };
}

/// It's the message you have to use in your sudo entrypoint,
/// the x/cwfees module sends these message as a sudo call to
/// your contract. Based on that information the contract
/// can decide if to accept the request, so return Ok, or
/// decline and return an error.
#[cw_serde]
#[non_exhaustive]
pub enum SudoMsg {
    CwGrant(CwGrant),
}

/// CwGrant is the only variant of the SudoMsg enum.
#[cw_serde]
pub struct CwGrant {
    /// Defines the amount of fees being requested for the execution of this tx.
    pub fee_requested: Vec<Coin>,
    /// Msgs contains the list of messages intended to be processed in this tx.
    pub msgs: Vec<Msg>,
}

/// Msg defines information about the tx messages.
/// It implements TryInto
#[cw_serde]
pub struct Msg {
    /// Defines the sender of the message, this is populated using the sdk.Msg.GetSigner()
    /// by the state machine. It can be trusted.
    pub sender: String,
    /// Defines the type_url of the message being sent, eg: /cosmos.bank.v1beta1.MsgSend.
    /// This can be used to decode the message to a specific type.
    pub type_url: String,
    /// Defines the binary representation of the message.
    pub msg: Binary,
}
impl Msg {
    /// Allows to convert Msg into a prost message. Note: all cosmos-sdk messages
    /// are prost messages.
    pub fn try_into_proto<T: prost::Message + Default>(self) -> Result<T, DecodeError> {
        T::decode(&*self.msg.0) //
    }
}

#[cfg(test)]
mod test {
    use crate::Msg;
    use cosmos_sdk_proto::cosmos::bank::v1beta1::MsgSend;
    use cosmwasm_std::Binary;
    use prost::Message;

    #[test]
    fn msg_from_protobuf_message() {
        let sdk_msg = MsgSend {
            from_address: "Kim Dokja".to_string(),
            to_address: "Yoo Joonghyuk".to_string(),
            amount: vec![],
        };
        let encoded = sdk_msg.encode_to_vec();

        let msg = Msg {
            sender: "".to_string(),
            type_url: "".to_string(),
            msg: Binary::from(encoded),
        };

        let got_sdk_msg: MsgSend = msg.try_into_proto().unwrap();
        assert_eq!(sdk_msg, got_sdk_msg)
    }
}
