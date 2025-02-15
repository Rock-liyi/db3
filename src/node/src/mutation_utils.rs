use bytes::Bytes;
use db3_crypto::db3_address::DB3Address;
use db3_crypto::db3_verifier;
use db3_crypto::id::AccountId;
use db3_error::DB3Error;
use db3_proto::db3_mutation_proto::{
    DatabaseMutation, MintCreditsMutation, PayloadType, WriteRequest,
};
use db3_proto::db3_mutation_v2_proto::Mutation as MutationV2;
use db3_proto::db3_session_proto::QuerySession;
use ethers::core::types::Bytes as EthersBytes;
use ethers::types::{
    transaction::eip712::{Eip712, TypedData},
    Address, Signature,
};
use prost::Message;
use std::str::FromStr;
use tracing::{info, warn};
/// parse mutation

macro_rules! parse_mutation {
    ($func:ident, $type:ident) => {
        pub fn $func(payload: &[u8]) -> Result<$type, DB3Error> {
            match $type::decode(payload) {
                Ok(dm) => match &dm.meta {
                    Some(_) => Ok(dm),
                    None => {
                        warn!("no meta for mutation");
                        Err(DB3Error::ApplyMutationError("meta is none".to_string()))
                    }
                },
                Err(e) => {
                    //TODO add event ?
                    warn!("invalid mutation data {e}");
                    Err(DB3Error::ApplyMutationError(
                        "invalid mutation data".to_string(),
                    ))
                }
            }
        }
    };
}

pub struct MutationUtil {}

impl MutationUtil {
    parse_mutation!(parse_database_mutation, DatabaseMutation);
    parse_mutation!(parse_mint_credits_mutation, MintCreditsMutation);
    parse_mutation!(parse_query_session, QuerySession);

    pub fn get_str_field<'a>(data: &'a TypedData, name: &'a str, default_val: &'a str) -> &'a str {
        if let Some(v) = data.message.get(name) {
            if let Some(t) = v.as_str() {
                t
            } else {
                default_val
            }
        } else {
            default_val
        }
    }

    pub fn get_u64_field(data: &TypedData, name: &str, default_val: u64) -> u64 {
        if let Some(v) = data.message.get(name) {
            if let Some(t) = v.as_str() {
                if let Ok(vt) = t.parse::<u64>() {
                    return vt;
                } else {
                    default_val
                }
            } else {
                default_val
            }
        } else {
            default_val
        }
    }

    pub fn verify_setup(payload: &[u8], sig: &str) -> Result<(Address, TypedData), DB3Error> {
        match serde_json::from_slice::<TypedData>(payload) {
            Ok(data) => {
                let signature = Signature::from_str(sig).map_err(|e| {
                    DB3Error::ApplyMutationError(format!("invalid signature for err {e}"))
                })?;
                let address = signature.recover_typed_data(&data).map_err(|e| {
                    DB3Error::ApplyMutationError(format!("invalid typed data for err {e}"))
                })?;
                Ok((address, data))
            }
            Err(e) => Err(DB3Error::ApplyMutationError(format!(
                "bad typed data for err {e}"
            ))),
        }
    }
    /// unwrap and verify write request
    pub fn unwrap_and_light_verify(
        payload: &[u8],
        sig: &str,
    ) -> Result<(MutationV2, DB3Address, u64), DB3Error> {
        match serde_json::from_slice::<TypedData>(payload) {
            Ok(data) => {
                // serde signature
                let signature = Signature::from_str(sig).map_err(|e| {
                    DB3Error::ApplyMutationError(format!("invalid signature for err {e}"))
                })?;
                if let (Some(payload), Some(nonce)) =
                    (data.message.get("payload"), data.message.get("nonce"))
                {
                    let address = signature.recover_typed_data(&data).map_err(|e| {
                        DB3Error::ApplyMutationError(format!("invalid typed data for err {e}"))
                    })?;
                    let db3_address = DB3Address::from(address.as_fixed_bytes());
                    let data: EthersBytes =
                        serde_json::from_value(payload.clone()).map_err(|e| {
                            DB3Error::ApplyMutationError(format!("invalid payload for err {e}"))
                        })?;
                    let dm = MutationV2::decode(data.as_ref()).map_err(|e| {
                        DB3Error::ApplyMutationError(format!("invalid mutation for err {e}"))
                    })?;
                    let real_nonce = u64::from_str(
                        nonce
                            .as_str()
                            .ok_or(DB3Error::ApplyMutationError("invalid nonce".to_string()))?,
                    )
                    .map_err(|e| {
                        DB3Error::ApplyMutationError(format!(
                            "fail to convert payload type to i32 {e}"
                        ))
                    })?;
                    Ok((dm, db3_address, real_nonce))
                } else {
                    Err(DB3Error::ApplyMutationError("bad typed data".to_string()))
                }
            }
            Err(e) => Err(DB3Error::ApplyMutationError(format!(
                "bad typed data for err {e}"
            ))),
        }
    }

    pub fn unwrap_and_verify(
        req: WriteRequest,
    ) -> Result<(EthersBytes, PayloadType, AccountId), DB3Error> {
        if req.payload_type == 3 {
            // typed data
            match serde_json::from_slice::<TypedData>(req.payload.as_ref()) {
                Ok(data) => {
                    let hashed_message = data.encode_eip712().map_err(|e| {
                        DB3Error::ApplyMutationError(format!("invalid payload type for err {e}"))
                    })?;
                    let account_id = db3_verifier::DB3Verifier::verify_hashed(
                        &hashed_message,
                        req.signature.as_ref(),
                    )?;
                    if let (Some(payload), Some(payload_type)) =
                        (data.message.get("payload"), data.message.get("payloadType"))
                    {
                        //TODO advoid data copy
                        let data: EthersBytes =
                            serde_json::from_value(payload.clone()).map_err(|e| {
                                DB3Error::ApplyMutationError(format!(
                                    "invalid payload type for err {e}"
                                ))
                            })?;
                        let internal_data_type = i32::from_str(payload_type.as_str().ok_or(
                            DB3Error::QuerySessionVerifyError("invalid payload type".to_string()),
                        )?)
                        .map_err(|e| {
                            DB3Error::QuerySessionVerifyError(format!(
                                "fail to convert payload type to i32 {e}"
                            ))
                        })?;
                        let data_type: PayloadType = PayloadType::from_i32(internal_data_type)
                            .ok_or(DB3Error::ApplyMutationError(
                                "invalid payload type".to_string(),
                            ))?;
                        Ok((data, data_type, account_id))
                    } else {
                        Err(DB3Error::ApplyMutationError("bad typed data".to_string()))
                    }
                }
                Err(e) => Err(DB3Error::ApplyMutationError(format!(
                    "bad typed data for err {e}"
                ))),
            }
        } else {
            let account_id =
                db3_verifier::DB3Verifier::verify(req.payload.as_ref(), req.signature.as_ref())?;
            let data_type: PayloadType = PayloadType::from_i32(req.payload_type).ok_or(
                DB3Error::ApplyMutationError("invalid payload type".to_string()),
            )?;
            let data = Bytes::from(req.payload);
            Ok((EthersBytes(data), data_type, account_id))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    use chrono::Utc;
    use db3_crypto::db3_signer::Db3MultiSchemeSigner;
    use db3_proto::db3_base_proto::{BroadcastMeta, ChainId, ChainRole};
    use db3_proto::db3_mutation_proto::DatabaseAction;
    fn create_a_database_mutation() -> DatabaseMutation {
        let meta = BroadcastMeta {
            //TODO get from network
            nonce: Utc::now().timestamp() as u64,
            //TODO use config
            chain_id: ChainId::DevNet.into(),
            //TODO use config
            chain_role: ChainRole::StorageShardChain.into(),
        };
        let dm = DatabaseMutation {
            meta: Some(meta),
            collection_mutations: vec![],
            db_address: vec![],
            action: DatabaseAction::CreateDb.into(),
            document_mutations: vec![],
            db_desc: "".to_string(),
        };
        dm
    }
    #[test]
    pub fn unwrap_and_verify_ut() {
        let kp = db3_cmd::keystore::KeyStore::get_keypair(None).unwrap();
        let signer = Db3MultiSchemeSigner::new(kp);
        let dm = create_a_database_mutation();
        let mut mbuf = BytesMut::with_capacity(1024 * 4);
        dm.encode(&mut mbuf).unwrap();
        let mbuf = mbuf.freeze();
        let signature = signer.sign(mbuf.as_ref()).unwrap();
        let request = WriteRequest {
            signature: signature.as_ref().to_vec(),
            payload: mbuf.as_ref().to_vec().to_owned(),
            payload_type: PayloadType::DatabasePayload.into(),
        };
        let (_, payload_type, _) = MutationUtil::unwrap_and_verify(request).unwrap();
        assert_eq!(PayloadType::DatabasePayload, payload_type);
    }
    #[test]
    pub fn test_java_sdk_verfiy_ut() {
        //let expected_addr = "f39fd6e51aad88f6f4ce6ab8827279cfffb92266";
        let typed_data = r#"
       {"types":{"EIP712Domain":[{"name":"name","type":"string"}],"Message":[{"name":"payload","type":"bytes"},{"name":"nonce","type":"string"}]},"primaryType":"Message","message":{"payload":"0x1a0822060a0464657363","nonce":"1"},"domain":{"name":"db3.network"}}
        "#;
        let typed_data_obj = serde_json::from_slice::<TypedData>(typed_data.as_bytes()).unwrap();
        let hashed_message = typed_data_obj.encode_eip712().unwrap();
        let hex_str = hex::encode(hashed_message);
        assert_eq!(
            "2b6ab2777e1ffb472f2f3206566f0cb691228ba5fb02692fd8fe933576b5003e",
            hex_str.as_str()
        );
    }
}
