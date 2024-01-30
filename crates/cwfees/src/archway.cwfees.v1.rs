/// MsgRegisterAsGranter allows a contract to register itself as a fee granter.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRegisterAsGranter {
    #[prost(string, tag = "1")]
    pub granting_contract: ::prost::alloc::string::String,
}
/// MsgUnregisterAsGranter can be used by a cosmwasm contract to unregister itself as a fee granter.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgUnregisterAsGranter {
    #[prost(string, tag = "1")]
    pub granting_contract: ::prost::alloc::string::String,
}
