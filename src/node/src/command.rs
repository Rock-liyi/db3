//
// command.rs
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

use crate::indexer_impl::IndexerNodeImpl;
use crate::rollup_executor::RollupExecutorConfig;
use crate::storage_node_light_impl::{StorageNodeV2Config, StorageNodeV2Impl};
use clap::Parser;
use db3_cmd::command::{DB3ClientCommand, DB3ClientContext, DB3ClientContextV2};
use db3_crypto::db3_address::DB3Address;
use db3_crypto::db3_signer::Db3MultiSchemeSigner;
use db3_proto::db3_indexer_proto::indexer_node_server::IndexerNodeServer;
use db3_proto::db3_node_proto::storage_node_client::StorageNodeClient;
use db3_proto::db3_storage_proto::storage_node_client::StorageNodeClient as StorageNodeV2Client;
use db3_proto::db3_storage_proto::storage_node_server::StorageNodeServer as StorageNodeV2Server;
use db3_proto::db3_storage_proto::{
    EventMessage as EventMessageV2, Subscription as SubscriptionV2,
};
use db3_sdk::mutation_sdk::MutationSDK;
use db3_sdk::store_sdk::StoreSDK;
use db3_sdk::store_sdk_v2::StoreSDKV2;
use db3_storage::db_store_v2::DBStoreV2Config;
use db3_storage::doc_store::DocStoreConfig;
use db3_storage::mutation_store::MutationStoreConfig;
use db3_storage::state_store::StateStoreConfig;
use http::Uri;
use std::io::{stderr, stdout};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tonic::codegen::http::Method;
use tonic::transport::{ClientTlsConfig, Endpoint, Server};
use tonic::Status;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::filter::LevelFilter;

const ABOUT: &str = "
██████╗ ██████╗ ██████╗ 
██╔══██╗██╔══██╗╚════██╗
██║  ██║██████╔╝ █████╔╝
██║  ██║██╔══██╗ ╚═══██╗
██████╔╝██████╔╝██████╔╝
╚═════╝ ╚═════╝ ╚═════╝ 
@db3.network🚀🚀🚀";

#[derive(Debug, Parser)]
#[clap(name = "db3")]
#[clap(about = ABOUT, long_about = None)]
pub enum DB3Command {
    /// Start the store node
    #[clap(name = "store")]
    Store {
        /// Bind the gprc server to this .
        #[clap(long, default_value = "127.0.0.1")]
        public_host: String,
        /// The port of grpc api
        #[clap(long, default_value = "26619")]
        public_grpc_port: u16,
        /// Log more logs
        #[clap(short, long)]
        verbose: bool,
        /// The database path for mutation
        #[clap(long, default_value = "./mutation_db")]
        mutation_db_path: String,
        /// The database path for state
        #[clap(long, default_value = "./state_db")]
        state_db_path: String,
        /// The database path for doc db
        #[clap(long, default_value = "./doc_db")]
        doc_db_path: String,
        /// The network id
        #[clap(long, default_value = "10")]
        network_id: u64,
        /// The block interval
        #[clap(long, default_value = "2000")]
        block_interval: u64,
        /// The interval of rollup
        #[clap(long, default_value = "60000")]
        rollup_interval: u64,
        /// The min data byte size for rollup
        #[clap(long, default_value = "102400")]
        rollup_min_data_size: u64,
        /// The data path of rollup
        #[clap(long, default_value = "./rollup_data")]
        rollup_data_path: String,
        /// The Ar miner node
        #[clap(long, default_value = "http://127.0.0.1:1984/")]
        ar_node_url: String,
        /// The Ar wallet path
        #[clap(long, default_value = "./keys")]
        key_root_path: String,
        /// The min gc round offset
        #[clap(long, default_value = "8")]
        min_gc_round_offset: u64,
        #[clap(long, default_value = "0x7b68E10c80474DD93bD8C1ad53D4463c60a3AB7c")]
        contract_addr: String,
        #[clap(long, default_value = "http://127.0.0.1:8545")]
        evm_node_url: String,
        /// the admin address which can change the configuration this node
        #[clap(long, default_value = "0x0000000000000000000000000000000000000000")]
        admin_addr: String,
    },

    /// Start db3 interactive console
    #[clap(name = "console")]
    Console {
        /// the url of db3 grpc api
        #[clap(long = "url", global = true, default_value = "http://127.0.0.1:26659")]
        public_grpc_url: String,
    },

    /// Start db3 indexer
    #[clap(name = "indexer")]
    Indexer {
        /// Bind the gprc server to this .
        #[clap(long, default_value = "127.0.0.1")]
        public_host: String,
        /// The port of grpc api
        #[clap(long, default_value = "26639")]
        public_grpc_port: u16,
        /// the store grpc url
        #[clap(
            long = "db3_storage_grpc_url",
            default_value = "http://127.0.0.1:26619"
        )]
        db3_storage_grpc_url: String,
        #[clap(short, long, default_value = "./index_meta_db")]
        meta_db_path: String,
        #[clap(short, long, default_value = "./index_doc_db")]
        doc_db_path: String,
        #[clap(short, long, default_value = "./keys")]
        key_root_path: String,
        #[clap(
            short,
            long,
            default_value = "0x7b68E10c80474DD93bD8C1ad53D4463c60a3AB7c"
        )]
        contract_addr: String,
        #[clap(
            short,
            long,
            default_value = "0x0000000000000000000000000000000000000000"
        )]
        admin_addr: String,
        #[clap(long, default_value = "http://127.0.0.1:8545")]
        evm_node_url: String,
        #[clap(long, default_value = "10")]
        network_id: u64,
        #[clap(short, long)]
        verbose: bool,
    },

    /// Run db3 client
    #[clap(name = "client")]
    Client {
        /// the url of db3 grpc api
        #[clap(long = "url", global = true, default_value = "http://127.0.0.1:26659")]
        public_grpc_url: String,
        /// the subcommand
        #[clap(subcommand)]
        cmd: Option<DB3ClientCommand>,
    },
}

impl DB3Command {
    fn build_context(public_grpc_url: &str) -> DB3ClientContext {
        let uri = public_grpc_url.parse::<Uri>().unwrap();
        let endpoint = match uri.scheme_str() == Some("https") {
            true => {
                let rpc_endpoint = Endpoint::new(public_grpc_url.to_string())
                    .unwrap()
                    .tls_config(ClientTlsConfig::new())
                    .unwrap();
                rpc_endpoint
            }
            false => {
                let rpc_endpoint = Endpoint::new(public_grpc_url.to_string()).unwrap();
                rpc_endpoint
            }
        };
        let channel = endpoint.connect_lazy();
        let node = Arc::new(StorageNodeClient::new(channel));
        if !db3_cmd::keystore::KeyStore::has_key(None) {
            db3_cmd::keystore::KeyStore::recover_keypair(None).unwrap();
        }
        let kp = db3_cmd::keystore::KeyStore::get_keypair(None).unwrap();
        let signer = Db3MultiSchemeSigner::new(kp);
        let mutation_sdk = MutationSDK::new(node.clone(), signer, true);
        let kp = db3_cmd::keystore::KeyStore::get_keypair(None).unwrap();
        let signer = Db3MultiSchemeSigner::new(kp);
        let store_sdk = StoreSDK::new(node, signer, true);
        DB3ClientContext {
            mutation_sdk: Some(mutation_sdk),
            store_sdk: Some(store_sdk),
        }
    }
    fn build_context_v2(public_grpc_url: &str) -> DB3ClientContextV2 {
        let uri = public_grpc_url.parse::<Uri>().unwrap();
        let endpoint = match uri.scheme_str() == Some("https") {
            true => {
                let rpc_endpoint = Endpoint::new(public_grpc_url.to_string())
                    .unwrap()
                    .tls_config(ClientTlsConfig::new())
                    .unwrap();
                rpc_endpoint
            }
            false => {
                let rpc_endpoint = Endpoint::new(public_grpc_url.to_string()).unwrap();
                rpc_endpoint
            }
        };
        let channel = endpoint.connect_lazy();
        let node = Arc::new(StorageNodeV2Client::new(channel));
        if !db3_cmd::keystore::KeyStore::has_key(None) {
            db3_cmd::keystore::KeyStore::recover_keypair(None).unwrap();
        }
        let kp = db3_cmd::keystore::KeyStore::get_keypair(None).unwrap();
        let signer = Db3MultiSchemeSigner::new(kp);
        let store_sdk = StoreSDKV2::new(node, signer);
        DB3ClientContextV2 {
            store_sdk: Some(store_sdk),
        }
    }

    pub async fn execute(self) {
        match self {
            DB3Command::Store {
                public_host,
                public_grpc_port,
                verbose,
                mutation_db_path,
                state_db_path,
                doc_db_path,
                network_id,
                block_interval,
                rollup_interval,
                rollup_min_data_size,
                rollup_data_path,
                ar_node_url,
                key_root_path,
                min_gc_round_offset,
                contract_addr,
                evm_node_url,
                admin_addr,
            } => {
                let log_level = if verbose {
                    LevelFilter::DEBUG
                } else {
                    LevelFilter::INFO
                };
                tracing_subscriber::fmt().with_max_level(log_level).init();
                info!("{ABOUT}");
                Self::start_store_grpc_service(
                    public_host.as_str(),
                    public_grpc_port,
                    mutation_db_path.as_str(),
                    state_db_path.as_str(),
                    doc_db_path.as_str(),
                    network_id,
                    block_interval,
                    rollup_interval,
                    rollup_min_data_size,
                    rollup_data_path.as_str(),
                    ar_node_url.as_str(),
                    key_root_path.as_str(),
                    min_gc_round_offset,
                    contract_addr.as_str(),
                    evm_node_url.as_str(),
                    admin_addr.as_str(),
                )
                .await;
                let running = Arc::new(AtomicBool::new(true));
                let r = running.clone();
                ctrlc::set_handler(move || {
                    r.store(false, Ordering::SeqCst);
                })
                .expect("Error setting Ctrl-C handler");
                loop {
                    if running.load(Ordering::SeqCst) {
                        let ten_millis = Duration::from_millis(10);
                        thread::sleep(ten_millis);
                    } else {
                        info!("stop db3 store node...");
                        break;
                    }
                }
            }

            DB3Command::Console { public_grpc_url } => {
                let ctx = Self::build_context(public_grpc_url.as_ref());
                db3_cmd::console::start_console(ctx, &mut stdout(), &mut stderr())
                    .await
                    .unwrap();
            }

            DB3Command::Indexer {
                public_host,
                public_grpc_port,
                db3_storage_grpc_url,
                meta_db_path,
                doc_db_path,
                key_root_path,
                contract_addr,
                evm_node_url,
                network_id,
                verbose,
                admin_addr,
            } => {
                let log_level = if verbose {
                    LevelFilter::DEBUG
                } else {
                    LevelFilter::INFO
                };

                tracing_subscriber::fmt().with_max_level(log_level).init();
                info!("{ABOUT}");

                let ctx = Self::build_context_v2(db3_storage_grpc_url.as_ref());

                let doc_store_conf = DocStoreConfig {
                    db_root_path: doc_db_path,
                    in_memory_db_handle_limit: 16,
                };
                let db_store_config = DBStoreV2Config {
                    db_path: meta_db_path.to_string(),
                    db_store_cf_name: "db_store_cf".to_string(),
                    doc_store_cf_name: "doc_store_cf".to_string(),
                    collection_store_cf_name: "col_store_cf".to_string(),
                    index_store_cf_name: "idx_store_cf".to_string(),
                    doc_owner_store_cf_name: "doc_owner_store_cf".to_string(),
                    db_owner_store_cf_name: "db_owner_cf".to_string(),
                    scan_max_limit: 1000,
                    enable_doc_store: true,
                    doc_store_conf,
                };
                let addr = format!("{public_host}:{public_grpc_port}");
                let indexer = IndexerNodeImpl::new(
                    db_store_config,
                    network_id,
                    addr.to_string(),
                    key_root_path,
                    contract_addr,
                    evm_node_url,
                    admin_addr,
                )
                .unwrap();
                let indexer_for_syncing = indexer.clone();
                let listen = tokio::spawn(async move {
                    info!("start syncing data from storage node");
                    indexer_for_syncing
                        .start(ctx.store_sdk.unwrap())
                        .await
                        .unwrap();
                });
                info!("start db3 indexer node on public addr {}", addr);
                let cors_layer = CorsLayer::new()
                    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                    .allow_headers(Any)
                    .allow_origin(Any);
                Server::builder()
                    .accept_http1(true)
                    .layer(cors_layer)
                    .layer(tonic_web::GrpcWebLayer::new())
                    .add_service(IndexerNodeServer::new(indexer))
                    .serve(addr.parse().unwrap())
                    .await
                    .unwrap();
                let (r1,) = tokio::join!(listen);
                r1.unwrap();
                info!("exit standalone indexer")
            }

            DB3Command::Client {
                cmd,
                public_grpc_url,
            } => {
                let mut ctx = Self::build_context(public_grpc_url.as_ref());
                if let Some(c) = cmd {
                    match c.execute(&mut ctx).await {
                        Ok(table) => table.printstd(),
                        Err(e) => println!("{}", e),
                    }
                }
            }
        }
    }
    /// Start store grpc service
    async fn start_store_grpc_service(
        public_host: &str,
        public_grpc_port: u16,
        mutation_db_path: &str,
        state_db_path: &str,
        doc_db_path: &str,
        network_id: u64,
        block_interval: u64,
        rollup_interval: u64,
        rollup_min_data_size: u64,
        rollup_data_path: &str,
        ar_node_url: &str,
        key_root_path: &str,
        min_gc_round_offset: u64,
        contract_addr: &str,
        evm_node_url: &str,
        admin_addr: &str,
    ) {
        let addr = format!("{public_host}:{public_grpc_port}");

        let rollup_config = RollupExecutorConfig {
            rollup_interval,
            temp_data_path: rollup_data_path.to_string(),
            ar_node_url: ar_node_url.to_string(),
            key_root_path: key_root_path.to_string(),
            min_rollup_size: rollup_min_data_size,
            min_gc_round_offset,
            evm_node_url: evm_node_url.to_string(),
            contract_addr: contract_addr.to_string(),
        };

        let store_config = MutationStoreConfig {
            db_path: mutation_db_path.to_string(),
            block_store_cf_name: "block_store_cf".to_string(),
            tx_store_cf_name: "tx_store_cf".to_string(),
            rollup_store_cf_name: "rollup_store_cf".to_string(),
            gc_cf_name: "gc_store_cf".to_string(),
            message_max_buffer: 4 * 1024,
            scan_max_limit: 50,
            block_state_cf_name: "block_state_cf".to_string(),
        };

        let state_config = StateStoreConfig {
            db_path: state_db_path.to_string(),
        };

        let db_store_config = DBStoreV2Config {
            db_path: doc_db_path.to_string(),
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

        let (sender, receiver) = tokio::sync::mpsc::channel::<(
            DB3Address,
            SubscriptionV2,
            Sender<std::result::Result<EventMessageV2, Status>>,
        )>(1024);
        let config = StorageNodeV2Config {
            store_config,
            state_config,
            rollup_config,
            db_store_config,
            network_id,
            block_interval,
            node_url: addr.to_string(),
            contract_addr: contract_addr.to_string(),
            evm_node_url: evm_node_url.to_string(),
            admin_addr: admin_addr.to_string(),
        };
        let storage_node = StorageNodeV2Impl::new(config, sender).await.unwrap();
        info!(
            "start db3 store node on public addr {} and network {}",
            addr, network_id
        );
        std::fs::create_dir_all(rollup_data_path).unwrap();
        storage_node.keep_subscription(receiver).await.unwrap();
        storage_node.start_to_produce_block().await;
        storage_node.start_to_rollup().await;
        let cors_layer = CorsLayer::new()
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(Any)
            .allow_origin(Any);
        Server::builder()
            .accept_http1(true)
            .layer(cors_layer)
            .layer(tonic_web::GrpcWebLayer::new())
            .add_service(StorageNodeV2Server::new(storage_node))
            .serve(addr.parse().unwrap())
            .await
            .unwrap();
    }
}
