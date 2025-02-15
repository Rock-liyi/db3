//
// stroage_node_light_impl.rs
// Copyright (C) 2023 db3.network Author imotai <codego.me@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use crate::mutation_utils::MutationUtil;
use crate::rollup_executor::{RollupExecutor, RollupExecutorConfig};
use crate::version_util;
use db3_base::bson_util::bytes_to_bson_document;
use db3_base::strings;
use db3_crypto::db3_address::DB3Address;
use db3_crypto::db3_verifier::DB3Verifier;
use db3_crypto::id::TxId;
use db3_error::Result;
use db3_proto::db3_base_proto::{SystemConfig, SystemStatus};
use db3_proto::db3_mutation_v2_proto::{
    mutation::body_wrapper::Body, MutationAction, MutationRollupStatus,
};
use db3_proto::db3_storage_proto::block_response;
use db3_proto::db3_storage_proto::event_message::Event as EventV2;
use db3_proto::db3_storage_proto::{
    storage_node_server::StorageNode, BlockRequest, BlockResponse, ExtraItem,
    GetCollectionOfDatabaseRequest, GetCollectionOfDatabaseResponse, GetDatabaseOfOwnerRequest,
    GetDatabaseOfOwnerResponse, GetDatabaseRequest, GetDatabaseResponse, GetMutationBodyRequest,
    GetMutationBodyResponse, GetMutationHeaderRequest, GetMutationHeaderResponse,
    GetMutationStateRequest, GetMutationStateResponse, GetNonceRequest, GetNonceResponse,
    GetSystemStatusRequest, MutationStateView, ScanGcRecordRequest, ScanGcRecordResponse,
    ScanMutationHeaderRequest, ScanMutationHeaderResponse, ScanRollupRecordRequest,
    ScanRollupRecordResponse, SendMutationRequest, SendMutationResponse, SetupRequest,
    SetupResponse, SubscribeRequest,
};
use ethers::types::U256;

use db3_proto::db3_storage_proto::{
    BlockEvent as BlockEventV2, EventMessage as EventMessageV2, EventType as EventTypeV2,
    Subscription as SubscriptionV2,
};

use db3_storage::db_store_v2::{DBStoreV2, DBStoreV2Config};
use db3_storage::mutation_store::{MutationStore, MutationStoreConfig};
use db3_storage::state_store::{StateStore, StateStoreConfig};
use ethers::abi::Address;
use prost::Message;
use std::collections::{BTreeMap, HashSet};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender as BroadcastSender;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task;
use tokio::time::{sleep, Duration as TokioDuration};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{debug, info, warn};

#[derive(Clone)]
pub struct StorageNodeV2Config {
    pub store_config: MutationStoreConfig,
    pub state_config: StateStoreConfig,
    pub rollup_config: RollupExecutorConfig,
    pub db_store_config: DBStoreV2Config,
    pub network_id: u64,
    pub block_interval: u64,
    pub node_url: String,
    pub evm_node_url: String,
    pub contract_addr: String,
    pub admin_addr: String,
}

pub struct StorageNodeV2Impl {
    storage: MutationStore,
    state_store: StateStore,
    config: StorageNodeV2Config,
    running: Arc<AtomicBool>,
    db_store: DBStoreV2,
    sender: Sender<(
        DB3Address,
        SubscriptionV2,
        Sender<std::result::Result<EventMessageV2, Status>>,
    )>,
    broadcast_sender: BroadcastSender<EventMessageV2>,
    rollup_executor: Arc<RollupExecutor>,
    rollup_interval: Arc<AtomicU64>,
    network_id: Arc<AtomicU64>,
}

impl StorageNodeV2Impl {
    pub async fn new(
        config: StorageNodeV2Config,
        sender: Sender<(
            DB3Address,
            SubscriptionV2,
            Sender<std::result::Result<EventMessageV2, Status>>,
        )>,
    ) -> Result<Self> {
        if let Err(_e) = std::fs::create_dir_all(config.rollup_config.key_root_path.as_str()) {}
        let storage = MutationStore::new(config.store_config.clone())?;
        storage.recover()?;
        let state_store = StateStore::new(config.state_config.clone())?;
        let db_store = DBStoreV2::new(config.db_store_config.clone())?;
        let (broadcast_sender, _) = broadcast::channel(1024);
        let network_id = Arc::new(AtomicU64::new(config.network_id));
        let rollup_executor = Arc::new(
            RollupExecutor::new(
                config.rollup_config.clone(),
                storage.clone(),
                network_id.clone(),
            )
            .await?,
        );
        let rollup_interval = config.rollup_config.rollup_interval;
        Ok(Self {
            storage,
            state_store,
            config,
            running: Arc::new(AtomicBool::new(true)),
            db_store,
            sender,
            broadcast_sender,
            rollup_executor,
            rollup_interval: Arc::new(AtomicU64::new(rollup_interval)),
            network_id,
        })
    }

    pub fn recover(&self) -> Result<()> {
        let config = self.state_store.get_node_config("storage")?;
        if let Some(c) = config {
            self.rollup_executor
                .update_min_rollup_size(c.min_rollup_size);
            self.rollup_interval
                .store(c.rollup_interval, Ordering::Relaxed);
            self.network_id.store(c.network_id, Ordering::Relaxed);
            info!(
                "recover system config with min roll size {}, rollup interval {} and network id {}",
                c.min_rollup_size, c.rollup_interval, c.network_id
            );
        }
        Ok(())
    }

    pub async fn start_to_produce_block(&self) {
        let local_running = self.running.clone();
        let local_storage = self.storage.clone();
        let local_block_interval = self.config.block_interval;
        let local_event_sender = self.broadcast_sender.clone();
        task::spawn(async move {
            info!("start the block producer thread");
            while local_running.load(Ordering::Relaxed) {
                sleep(TokioDuration::from_millis(local_block_interval)).await;
                debug!(
                    "produce block {}",
                    local_storage.get_current_block().unwrap_or(0)
                );
                match local_storage.increase_block_return_last_state() {
                    Ok((block_id, mutation_count)) => {
                        // sender block event
                        let e = BlockEventV2 {
                            block_id,
                            mutation_count,
                        };
                        let msg = EventMessageV2 {
                            r#type: EventTypeV2::Block as i32,
                            event: Some(EventV2::BlockEvent(e)),
                        };
                        match local_event_sender.send(msg) {
                            Ok(_) => {
                                debug!("broadcast block event {}, {}", block_id, mutation_count);
                            }
                            Err(e) => {
                                warn!("the broadcast channel error for {:?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("fail to produce block for error {e}");
                    }
                }
            }
            info!("exit the block producer thread");
        });
    }

    pub async fn start_to_rollup(&self) {
        let local_running = self.running.clone();
        let executor = self.rollup_executor.clone();
        let rollup_interval = self.rollup_interval.clone();
        task::spawn(async move {
            info!("start the rollup thread");
            while local_running.load(Ordering::Relaxed) {
                sleep(TokioDuration::from_millis(
                    rollup_interval.load(Ordering::Relaxed),
                ))
                .await;
                match executor.process().await {
                    Ok(()) => {}
                    Err(e) => {
                        warn!("fail to rollup for error {e}");
                    }
                }
            }
            info!("exit the rollup thread");
        });
    }

    pub async fn keep_subscription(
        &self,
        mut receiver: Receiver<(
            DB3Address,
            SubscriptionV2,
            Sender<std::result::Result<EventMessageV2, Status>>,
        )>,
    ) -> std::result::Result<(), Status> {
        info!("start to keep subscription");
        let local_running = self.running.clone();
        let local_broadcast_sender = self.broadcast_sender.clone();

        tokio::spawn(async move {
            info!("listen to subscription update event and event message broadcaster");
            while local_running.load(Ordering::Relaxed) {
                info!("keep subscription loop");
                let mut subscribers: BTreeMap<
                    DB3Address,
                    (
                        Sender<std::result::Result<EventMessageV2, Status>>,
                        SubscriptionV2,
                    ),
                > = BTreeMap::new();
                let mut to_be_removed: HashSet<DB3Address> = HashSet::new();
                let mut event_sub = local_broadcast_sender.subscribe();
                while local_running.load(Ordering::Relaxed) {
                    tokio::select! {
                         Some((addr, sub, sender)) = receiver.recv() => {
                            info!("add or update the subscriber with addr 0x{}", hex::encode(addr.as_ref()));
                            //TODO limit the max address count
                            subscribers.insert(addr, (sender, sub));
                            info!("subscribers len : {}", subscribers.len());
                        }
                        Ok(event) = event_sub.recv() => {
                            debug!("receive event {:?}", event);
                            for (key , (sender, sub)) in subscribers.iter() {
                                if sender.is_closed() {
                                    to_be_removed.insert(key.clone());
                                    warn!("the channel has been closed by client for addr 0x{}", hex::encode(key.as_ref()));
                                    continue;
                                }
                                for idx in 0..sub.topics.len() {
                                    if sub.topics[idx] != EventTypeV2::Block as i32 {
                                        continue;
                                    }
                                    match sender.try_send(Ok(event.clone())) {
                                        Ok(_) => {
                                            debug!("send event to addr 0x{}", hex::encode(key.as_ref()));
                                            break;
                                        }
                                        Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                                            // retry?
                                            // TODO
                                            warn!("the channel is full for addr 0x{}", hex::encode(key.as_ref()));
                                        }
                                        Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                                            // remove the address
                                            to_be_removed.insert(key.clone());
                                            warn!("the channel has been closed by client for addr 0x{}", hex::encode(key.as_ref()));
                                        }

                                    }
                                }
                            }
                        },
                        else => {
                            info!("unexpected channel update");
                            // reconnect in 5 seconds
                            sleep(TokioDuration::from_millis(1000 * 5)).await;
                            break;
                        }

                    }
                    for k in to_be_removed.iter() {
                        subscribers.remove(k);
                    }
                    to_be_removed.clear();
                }
            }
            info!("exit the keep subscription thread");
        });
        Ok(())
    }
}

#[tonic::async_trait]
impl StorageNode for StorageNodeV2Impl {
    async fn get_mutation_state(
        &self,
        _request: Request<GetMutationStateRequest>,
    ) -> std::result::Result<Response<GetMutationStateResponse>, Status> {
        let state = self.storage.get_latest_state();
        let total_storage_cost = strings::ar_to_readable_num_str(U256::from_big_endian(
            state.total_storage_cost.as_ref() as &[u8],
        ));
        let total_evm_cost = strings::evm_to_readable_num_str(U256::from_big_endian(
            state.total_evm_cost.as_ref() as &[u8],
        ));
        let view = MutationStateView {
            mutation_count: state.mutation_count,
            total_mutation_bytes: state.total_mutation_bytes,
            gc_count: state.gc_count,
            rollup_count: state.rollup_count,
            total_rollup_bytes: state.total_rollup_bytes,
            total_gc_bytes: state.total_gc_bytes,
            total_rollup_raw_bytes: state.total_rollup_raw_bytes,
            total_rollup_mutation_count: state.total_rollup_mutation_count,
            total_storage_cost,
            total_evm_cost,
        };
        Ok(Response::new(GetMutationStateResponse { view: Some(view) }))
    }

    async fn setup(
        &self,
        request: Request<SetupRequest>,
    ) -> std::result::Result<Response<SetupResponse>, Status> {
        let r = request.into_inner();
        let (addr, data) = MutationUtil::verify_setup(&r.payload, r.signature.as_str())
            .map_err(|e| Status::internal(format!("{e}")))?;
        info!("setup the storage node with config {:?}", data);
        let admin_addr = self
            .config
            .admin_addr
            .parse::<Address>()
            .map_err(|e| Status::internal(format!("{e}")))?;
        if admin_addr != addr {
            return Err(Status::permission_denied(
                "You are not the admin".to_string(),
            ));
        }
        let rollup_interval = MutationUtil::get_u64_field(
            &data,
            "rollupInterval",
            self.rollup_interval.load(Ordering::Relaxed),
        );
        let min_rollup_size = MutationUtil::get_u64_field(
            &data,
            "minRollupSize",
            self.rollup_executor.get_min_rollup_size(),
        );
        let evm_node_rpc =
            MutationUtil::get_str_field(&data, "evmNodeRpc", self.config.evm_node_url.as_str());
        let ar_node_url = MutationUtil::get_str_field(
            &data,
            "arNodeUrl",
            self.config.rollup_config.ar_node_url.as_str(),
        );

        let network = MutationUtil::get_str_field(&data, "network", "0")
            .parse::<u64>()
            .map_err(|e| Status::internal(format!("{e}")))?;

        self.rollup_executor.update_min_rollup_size(min_rollup_size);
        self.rollup_interval
            .store(rollup_interval, Ordering::Relaxed);
        self.network_id.store(network, Ordering::Relaxed);
        let system_config = SystemConfig {
            min_rollup_size,
            rollup_interval,
            network_id: network,
            evm_node_url: evm_node_rpc.to_string(),
            ar_node_url: ar_node_url.to_string(),
        };
        self.state_store
            .store_node_config("storage", &system_config)
            .map_err(|e| Status::internal(format!("{e}")))?;
        return Ok(Response::new(SetupResponse {
            code: 0,
            msg: "ok".to_string(),
        }));
    }

    async fn get_system_status(
        &self,
        _request: Request<GetSystemStatusRequest>,
    ) -> std::result::Result<Response<SystemStatus>, Status> {
        let (addr, balance) = self
            .rollup_executor
            .get_ar_account()
            .await
            .map_err(|e| Status::internal(format!("{e}")))?;
        let evm_addr = self
            .rollup_executor
            .get_evm_account()
            .await
            .map_err(|e| Status::internal(format!("{e}")))?;
        let system_config = self
            .state_store
            .get_node_config("storage")
            .map_err(|e| Status::internal(format!("{e}")))?;
        let has_inited = !system_config.is_none();
        Ok(Response::new(SystemStatus {
            evm_account: evm_addr,
            evm_balance: "".to_string(),
            ar_account: addr,
            ar_balance: balance,
            node_url: self.config.node_url.to_string(),
            config: system_config,
            has_inited,
            admin_addr: self.config.admin_addr.to_string(),
            version: Some(version_util::build_version()),
        }))
    }

    async fn scan_gc_record(
        &self,
        request: Request<ScanGcRecordRequest>,
    ) -> std::result::Result<Response<ScanGcRecordResponse>, Status> {
        let r = request.into_inner();
        let records = self
            .storage
            .scan_gc_records(r.start, r.limit)
            .map_err(|e| Status::internal(format!("{e}")))?;
        Ok(Response::new(ScanGcRecordResponse { records }))
    }

    type SubscribeStream = ReceiverStream<std::result::Result<EventMessageV2, Status>>;
    /// add subscription to the light node
    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> std::result::Result<Response<Self::SubscribeStream>, Status> {
        info!("receive subscribe request");
        let r = request.into_inner();
        let sender = self.sender.clone();
        info!("sender is close: {}", sender.is_closed());
        let account_id = DB3Verifier::verify(r.payload.as_ref(), r.signature.as_ref())
            .map_err(|e| Status::internal(format!("bad signature for {e}")))?;
        let payload = SubscriptionV2::decode(r.payload.as_ref()).map_err(|e| {
            Status::internal(format!("fail to decode open session request for {e} "))
        })?;
        info!(
            "add subscriber for addr 0x{}",
            hex::encode(account_id.addr.as_ref())
        );
        info!("payload {:?}", payload);
        info!("sender {:?}", sender);
        let (msg_sender, msg_receiver) =
            tokio::sync::mpsc::channel::<std::result::Result<EventMessageV2, Status>>(10);
        sender
            .try_send((account_id.addr, payload, msg_sender))
            .map_err(|e| Status::internal(format!("fail to add subscriber for {e}")))?;
        Ok(Response::new(ReceiverStream::new(msg_receiver)))
    }

    async fn get_block(
        &self,
        request: Request<BlockRequest>,
    ) -> std::result::Result<Response<BlockResponse>, Status> {
        let r = request.into_inner();
        let mutation_header_bodys = self
            .storage
            .get_range_mutations(r.block_start, r.block_end)
            .map_err(|e| Status::internal(format!("{e}")))?;
        let mutations = mutation_header_bodys
            .iter()
            .map(|(h, b)| block_response::MutationWrapper {
                header: Some(h.to_owned()),
                body: Some(b.to_owned()),
            })
            .collect();
        Ok(Response::new(BlockResponse { mutations }))
    }

    async fn get_database(
        &self,
        request: Request<GetDatabaseRequest>,
    ) -> std::result::Result<Response<GetDatabaseResponse>, Status> {
        let r = request.into_inner();
        let addr = DB3Address::from_hex(r.addr.as_str())
            .map_err(|e| Status::invalid_argument(format!("invalid database address {e}")))?;
        let database = self
            .db_store
            .get_database(&addr)
            .map_err(|e| Status::internal(format!("fail to get database {e}")))?;
        let state = self.db_store.get_database_state(&addr);
        Ok(Response::new(GetDatabaseResponse { database, state }))
    }

    async fn get_collection_of_database(
        &self,
        request: Request<GetCollectionOfDatabaseRequest>,
    ) -> std::result::Result<Response<GetCollectionOfDatabaseResponse>, Status> {
        let r = request.into_inner();
        let addr = DB3Address::from_hex(r.db_addr.as_str())
            .map_err(|e| Status::invalid_argument(format!("invalid database address {e}")))?;
        let (collections, collection_states) = self
            .db_store
            .get_collection_of_database(&addr)
            .map_err(|e| Status::internal(format!("fail to get collect of database {e}")))?;
        info!(
            "query collection count {} with database {}",
            collections.len(),
            r.db_addr.as_str()
        );
        Ok(Response::new(GetCollectionOfDatabaseResponse {
            collections,
            states: collection_states,
        }))
    }

    async fn get_database_of_owner(
        &self,
        request: Request<GetDatabaseOfOwnerRequest>,
    ) -> std::result::Result<Response<GetDatabaseOfOwnerResponse>, Status> {
        let r = request.into_inner();
        let addr = DB3Address::from_hex(r.owner.as_str())
            .map_err(|e| Status::invalid_argument(format!("invalid database address {e}")))?;
        let (databases, states) = self
            .db_store
            .get_database_of_owner(&addr)
            .map_err(|e| Status::internal(format!("{e}")))?;
        info!(
            "query database list count {} with account {}",
            databases.len(),
            r.owner.as_str()
        );
        Ok(Response::new(GetDatabaseOfOwnerResponse {
            databases,
            states,
        }))
    }

    async fn get_mutation_body(
        &self,
        request: Request<GetMutationBodyRequest>,
    ) -> std::result::Result<Response<GetMutationBodyResponse>, Status> {
        let r = request.into_inner();
        let tx_id = TxId::try_from_hex(r.id.as_str())
            .map_err(|e| Status::invalid_argument(format!("invalid mutation id {e}")))?;
        let body = self
            .storage
            .get_mutation(&tx_id)
            .map_err(|e| Status::internal(format!("{e}")))?;
        Ok(Response::new(GetMutationBodyResponse { body }))
    }

    async fn scan_rollup_record(
        &self,
        request: Request<ScanRollupRecordRequest>,
    ) -> std::result::Result<Response<ScanRollupRecordResponse>, Status> {
        let r = request.into_inner();
        let mut records_pending = vec![self.rollup_executor.get_pending_rollup()];
        let records_done = self
            .storage
            .scan_rollup_records(r.start, r.limit)
            .map_err(|e| Status::internal(format!("{e}")))?;
        records_pending.extend_from_slice(&records_done);
        Ok(Response::new(ScanRollupRecordResponse {
            records: records_pending,
        }))
    }

    async fn scan_mutation_header(
        &self,
        request: Request<ScanMutationHeaderRequest>,
    ) -> std::result::Result<Response<ScanMutationHeaderResponse>, Status> {
        let r = request.into_inner();
        let headers = self
            .storage
            .scan_mutation_headers(r.start, r.limit)
            .map_err(|e| Status::internal(format!("{e}")))?;
        info!(
            "scan mutation headers {} with start {} and limit {}",
            headers.len(),
            r.start,
            r.limit
        );
        Ok(Response::new(ScanMutationHeaderResponse { headers }))
    }

    async fn get_mutation_header(
        &self,
        request: Request<GetMutationHeaderRequest>,
    ) -> std::result::Result<Response<GetMutationHeaderResponse>, Status> {
        let r = request.into_inner();
        let header = self
            .storage
            .get_mutation_header(r.block_id, r.order_id)
            .map_err(|e| Status::internal(format!("{e}")))?;
        Ok(Response::new(GetMutationHeaderResponse {
            header,
            status: MutationRollupStatus::Pending.into(),
            rollup_tx: vec![],
        }))
    }

    async fn get_nonce(
        &self,
        request: Request<GetNonceRequest>,
    ) -> std::result::Result<Response<GetNonceResponse>, Status> {
        let r = request.into_inner();
        let address = DB3Address::try_from(r.address.as_str())
            .map_err(|e| Status::invalid_argument(format!("invalid account address {e}")))?;
        let used_nonce = self
            .state_store
            .get_nonce(&address)
            .map_err(|e| Status::internal(format!("{e}")))?;
        info!("address {} used nonce {}", address.to_hex(), used_nonce);
        Ok(Response::new(GetNonceResponse {
            nonce: used_nonce + 1,
        }))
    }

    async fn send_mutation(
        &self,
        request: Request<SendMutationRequest>,
    ) -> std::result::Result<Response<SendMutationResponse>, Status> {
        let r = request.into_inner();
        let (dm, address, nonce) = MutationUtil::unwrap_and_light_verify(
            &r.payload,
            r.signature.as_str(),
        )
        .map_err(|e| {
            Status::invalid_argument(format!("fail to verify the payload and signature {e}"))
        })?;
        let action = MutationAction::from_i32(dm.action)
            .ok_or(Status::internal("fail to convert action type".to_string()))?;
        let network = self.network_id.load(Ordering::Relaxed);
        // TODO validate the database mutation
        match self.state_store.incr_nonce(&address, nonce) {
            Ok(_) => {
                // mutation id
                let (id, block, order) = self
                    .storage
                    .generate_mutation_block_and_order(&r.payload, r.signature.as_str())
                    .map_err(|e| Status::internal(format!("{e}")))?;
                let response = match action {
                    MutationAction::CreateEventDb => {
                        let mut items: Vec<ExtraItem> = Vec::new();
                        for body in dm.bodies {
                            if let Some(Body::EventDatabaseMutation(ref mutation)) = &body.body {
                                let db_id = self
                                    .db_store
                                    .create_event_database(
                                        &address, mutation, nonce, network, block, order,
                                    )
                                    .map_err(|e| Status::internal(format!("{e}")))?;
                                let db_id_hex = db_id.to_hex();
                                info!(
                                    "add database with addr {} from owner {}",
                                    db_id_hex.as_str(),
                                    address.to_hex().as_str()
                                );
                                let item = ExtraItem {
                                    key: "db_addr".to_string(),
                                    value: db_id_hex,
                                };
                                items.push(item);
                                break;
                            }
                        }
                        Response::new(SendMutationResponse {
                            id,
                            code: 0,
                            msg: "ok".to_string(),
                            items,
                            block,
                            order,
                        })
                    }
                    MutationAction::CreateDocumentDb => {
                        let mut items: Vec<ExtraItem> = Vec::new();
                        for body in dm.bodies {
                            if let Some(Body::DocDatabaseMutation(ref doc_db_mutation)) = &body.body
                            {
                                let db_id = self
                                    .db_store
                                    .create_doc_database(
                                        &address,
                                        doc_db_mutation,
                                        nonce,
                                        network,
                                        block,
                                        order,
                                    )
                                    .map_err(|e| Status::internal(format!("{e}")))?;
                                let db_id_hex = db_id.to_hex();
                                info!(
                                    "add database with addr {} from owner {}",
                                    db_id_hex.as_str(),
                                    address.to_hex().as_str()
                                );
                                let item = ExtraItem {
                                    key: "db_addr".to_string(),
                                    value: db_id_hex,
                                };
                                items.push(item);
                                break;
                            }
                        }
                        Response::new(SendMutationResponse {
                            id,
                            code: 0,
                            msg: "ok".to_string(),
                            items,
                            block,
                            order,
                        })
                    }
                    MutationAction::AddCollection => {
                        let mut items: Vec<ExtraItem> = Vec::new();
                        for (i, body) in dm.bodies.iter().enumerate() {
                            let db_address_ref: &[u8] = body.db_address.as_ref();
                            let db_addr = DB3Address::try_from(db_address_ref)
                                .map_err(|e| Status::internal(format!("{e}")))?;
                            if let Some(Body::CollectionMutation(ref col_mutation)) = &body.body {
                                self.db_store
                                    .create_collection(
                                        &address,
                                        &db_addr,
                                        col_mutation,
                                        block,
                                        order,
                                        i as u16,
                                    )
                                    .map_err(|e| Status::internal(format!("{e}")))?;
                                info!(
                                    "add collection with db_addr {}, collection_name: {}, from owner {}",
                                    db_addr.to_hex().as_str(),
                                    col_mutation.collection_name.as_str(),
                                    address.to_hex().as_str()
                                );
                                let item = ExtraItem {
                                    key: "collection".to_string(),
                                    value: col_mutation.collection_name.to_string(),
                                };
                                items.push(item);
                            }
                        }
                        Response::new(SendMutationResponse {
                            id,
                            code: 0,
                            msg: "ok".to_string(),
                            items,
                            block,
                            order,
                        })
                    }
                    MutationAction::AddDocument => {
                        let mut items: Vec<ExtraItem> = Vec::new();
                        for (_i, body) in dm.bodies.iter().enumerate() {
                            let db_address_ref: &[u8] = body.db_address.as_ref();
                            let db_addr = DB3Address::try_from(db_address_ref)
                                .map_err(|e| Status::internal(format!("{e}")))?;
                            if let Some(Body::DocumentMutation(ref doc_mutation)) = &body.body {
                                let mut docs = Vec::<String>::new();
                                for buf in doc_mutation.documents.iter() {
                                    let document = bytes_to_bson_document(buf.clone())
                                        .map_err(|e| Status::internal(format!("{e}")))?;
                                    docs.push(document.to_string());
                                }
                                let ids = self
                                    .db_store
                                    .add_docs(
                                        &db_addr,
                                        &address,
                                        doc_mutation.collection_name.as_str(),
                                        &docs,
                                    )
                                    .map_err(|e| Status::internal(format!("{e}")))?;
                                info!(
                                    "add documents with db_addr {}, collection_name: {}, from owner {}, document size: {}",
                                    db_addr.to_hex().as_str(),
                                    doc_mutation.collection_name.as_str(),
                                    address.to_hex().as_str(),
                                    ids.len()
                                );
                                // return document keys
                                for id in ids {
                                    let item = ExtraItem {
                                        key: "document".to_string(),
                                        value: id.to_string(),
                                    };
                                    items.push(item);
                                }
                            }
                        }
                        Response::new(SendMutationResponse {
                            id,
                            code: 0,
                            msg: "ok".to_string(),
                            items,
                            block,
                            order,
                        })
                    }
                    MutationAction::UpdateDocument => {
                        for (_i, body) in dm.bodies.iter().enumerate() {
                            let db_address_ref: &[u8] = body.db_address.as_ref();
                            let db_addr = DB3Address::try_from(db_address_ref)
                                .map_err(|e| Status::internal(format!("{e}")))?;
                            if let Some(Body::DocumentMutation(ref doc_mutation)) = &body.body {
                                if doc_mutation.documents.len() != doc_mutation.ids.len() {
                                    let msg = format!(
                                        "doc ids size {} not equal to documents size {}",
                                        doc_mutation.ids.len(),
                                        doc_mutation.documents.len()
                                    );
                                    warn!("{}", msg.as_str());
                                    return Err(Status::internal(msg));
                                }
                                let mut docs = Vec::<String>::new();
                                for buf in doc_mutation.documents.iter() {
                                    let document = bytes_to_bson_document(buf.clone())
                                        .map_err(|e| Status::internal(format!("{e}")))?;
                                    let doc_str = document.to_string();
                                    debug!("update document: {}", doc_str);
                                    docs.push(doc_str);
                                }
                                self.db_store
                                    .update_docs(
                                        &db_addr,
                                        &address,
                                        doc_mutation.collection_name.as_str(),
                                        &docs,
                                        &doc_mutation.ids,
                                    )
                                    .map_err(|e| Status::internal(format!("{e}")))?;
                                info!(
                                    "update documents with db_addr {}, collection_name: {}, from owner {}",
                                    db_addr.to_hex().as_str(),
                                    doc_mutation.collection_name.as_str(),
                                    address.to_hex().as_str()
                                );
                            }
                        }
                        Response::new(SendMutationResponse {
                            id,
                            code: 0,
                            msg: "ok".to_string(),
                            items: vec![],
                            block,
                            order,
                        })
                    }
                    MutationAction::DeleteDocument => {
                        for (_i, body) in dm.bodies.iter().enumerate() {
                            let db_address_ref: &[u8] = body.db_address.as_ref();
                            let db_addr = DB3Address::try_from(db_address_ref)
                                .map_err(|e| Status::internal(format!("{e}")))?;
                            if let Some(Body::DocumentMutation(ref doc_mutation)) = &body.body {
                                self.db_store
                                    .delete_docs(
                                        &db_addr,
                                        &address,
                                        doc_mutation.collection_name.as_str(),
                                        &doc_mutation.ids,
                                    )
                                    .map_err(|e| Status::internal(format!("{e}")))?;
                                info!(
                                    "delete documents with db_addr {}, collection_name: {}, from owner {}",
                                    db_addr.to_hex().as_str(),
                                    doc_mutation.collection_name.as_str(),
                                    address.to_hex().as_str()
                                );
                            }
                        }
                        Response::new(SendMutationResponse {
                            id,
                            code: 0,
                            msg: "ok".to_string(),
                            items: vec![],
                            block,
                            order,
                        })
                    }
                };
                self.storage
                    .add_mutation(
                        &r.payload,
                        r.signature.as_str(),
                        &address,
                        nonce,
                        block,
                        order,
                        network,
                        action,
                    )
                    .map_err(|e| Status::internal(format!("{e}")))?;
                Ok(response)
            }
            Err(_e) => Ok(Response::new(SendMutationResponse {
                id: "".to_string(),
                code: 1,
                msg: "bad nonce".to_string(),
                items: vec![],
                block: 0,
                order: 0,
            })),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use db3_storage::doc_store::DocStoreConfig;
    use tempdir::TempDir;

    fn generate_rand_node_config() -> StorageNodeV2Config {
        let tmp_dir_path = TempDir::new("add_store_path").expect("create temp dir");
        let real_path = tmp_dir_path.path().to_str().unwrap().to_string();
        let rollup_config = RollupExecutorConfig {
            rollup_interval: 1000000,
            temp_data_path: format!("{real_path}/data_path"),
            ar_node_url: "http://127.0.0.1:1984".to_string(),
            key_root_path: format!("{real_path}/keys"),
            min_rollup_size: 1000000,
            min_gc_round_offset: 100,
            evm_node_url: "http://127.0.0.1:8545".to_string(),
            contract_addr: "0x5FbDB2315678afecb367f032d93F642f64180aa3".to_string(),
        };
        let store_config = MutationStoreConfig {
            db_path: format!("{real_path}/mutation_path"),
            block_store_cf_name: "block_store_cf".to_string(),
            tx_store_cf_name: "tx_store_cf".to_string(),
            rollup_store_cf_name: "rollup_store_cf".to_string(),
            gc_cf_name: "gc_store_cf".to_string(),
            message_max_buffer: 4 * 1024,
            scan_max_limit: 50,
            block_state_cf_name: "block_state_cf".to_string(),
        };
        let state_config = StateStoreConfig {
            db_path: format!("{real_path}/state_store"),
        };
        let db_store_config = DBStoreV2Config {
            db_path: format!("{real_path}/db_path").to_string(),
            db_store_cf_name: "db_store_cf".to_string(),
            doc_store_cf_name: "doc_store_cf".to_string(),
            collection_store_cf_name: "col_store_cf".to_string(),
            index_store_cf_name: "idx_store_cf".to_string(),
            doc_owner_store_cf_name: "doc_owner_store_cf".to_string(),
            db_owner_store_cf_name: "db_owner_cf".to_string(),
            scan_max_limit: 1000,
            enable_doc_store: false,
            doc_store_conf: DocStoreConfig::default(),
        };
        StorageNodeV2Config {
            store_config,
            state_config,
            rollup_config,
            db_store_config,
            network_id: 10,
            block_interval: 10000,
            node_url: "http://127.0.0.1:26639".to_string(),
            contract_addr: "0x5FbDB2315678afecb367f032d93F642f64180aa3".to_string(),
            evm_node_url: "http://127.0.0.1:8545".to_string(),
            admin_addr: "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266".to_string(),
        }
    }
    #[tokio::test]
    async fn test_storage_node_recover() {
        let (sender, _) = tokio::sync::mpsc::channel::<(
            DB3Address,
            SubscriptionV2,
            Sender<std::result::Result<EventMessageV2, Status>>,
        )>(1024);
        let config = generate_rand_node_config();
        let sig:&str = "0x484f7b5353c5238ea1c6a1ab4552cc61b726b10ee3b68fb6be717e1f6fa37f5e248889ab334ee91b83f1250576b6c1faf068d3498f294eb77b600c0115b5919c1c";
        let payload:&str = "7b227479706573223a7b22454950373132446f6d61696e223a5b5d2c224d657373616765223a5b7b226e616d65223a22726f6c6c7570496e74657276616c222c2274797065223a22737472696e67227d2c7b226e616d65223a226d696e526f6c6c757053697a65222c2274797065223a22737472696e67227d2c7b226e616d65223a226e6574776f726b222c2274797065223a22737472696e67227d5d7d2c22646f6d61696e223a7b7d2c227072696d61727954797065223a224d657373616765222c226d657373616765223a7b22726f6c6c7570496e74657276616c223a2231303030303030222c226d696e526f6c6c757053697a65223a2231303030303030222c226e6574776f726b223a2239353237227d7d";
        match StorageNodeV2Impl::new(config.clone(), sender).await {
            Ok(storage_node) => {
                let payload_binary = hex::decode(payload).unwrap();
                let request = SetupRequest {
                    signature: sig.to_string(),
                    payload: payload_binary,
                };
                let tonic_req = Request::new(request);
                if let Ok(response) = storage_node.setup(tonic_req).await {
                    let r = response.into_inner();
                    assert_eq!(0, r.code);
                } else {
                    assert!(false);
                }
            }
            Err(e) => {
                println!("{e}");
                assert!(false);
            }
        }
        let (sender, _) = tokio::sync::mpsc::channel::<(
            DB3Address,
            SubscriptionV2,
            Sender<std::result::Result<EventMessageV2, Status>>,
        )>(1024);
        if let Ok(storage_node) = StorageNodeV2Impl::new(config.clone(), sender).await {
            if let Err(_e) = storage_node.recover() {
                assert!(false);
            }
            let tonic_req = Request::new(GetSystemStatusRequest {});
            if let Ok(response) = storage_node.get_system_status(tonic_req).await {
                let r = response.into_inner();
                assert!(r.has_inited);
                let config = r.config.unwrap();
                assert_eq!(config.min_rollup_size, 1000000);
                assert_eq!(config.network_id, 9527);
                assert_eq!(config.rollup_interval, 1000000);
            }
        } else {
            assert!(false);
        }
    }
}
