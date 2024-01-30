# MsgAllowlist

This contract implements a fee granting strategy in which we allow a list of users
to only perform contract executions towards an allowed contract.

The use case is enabling a set of users to only spend granted fees on a specific contract,
it can be specialised further for fees to be spent on specific methods of the contract.
We showcase the extreme flexibility of the x/cwfees module.

NOTE: importing cosmos_sdk_proto will not make compile in CW, so you'll need to manually build them using prost::build, 
you have in example in the [cwfees package](../../crates/cwfees/build.rs)
