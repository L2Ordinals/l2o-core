pub mod error;
pub mod server;
pub mod store;

use std::sync::Arc;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::web;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use async_channel::Receiver;
use bitcoincore_rpc::bitcoin::Transaction;
use indexer_sdk::client::drect::DirectClient;
use indexer_sdk::client::event::ClientEvent;
use indexer_sdk::client::Client;
use indexer_sdk::configuration::base::IndexerConfiguration;
use indexer_sdk::configuration::base::NetConfiguration;
use indexer_sdk::configuration::base::ZMQConfiguration;
use indexer_sdk::event::TxIdType;
use indexer_sdk::factory::common::async_create_and_start_processor;
use indexer_sdk::storage::StorageProcessor;
use indexer_sdk::types::delta::TransactionDelta;
use indexer_sdk::types::response::GetDataResponse;
use tokio::runtime::Runtime;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::sleep;
use tokio::time::Duration;

use crate::error::Result;

pub async fn listen() -> Result<()> {
    let (tx, rx) = watch::channel(());
    let rt = Arc::new(Runtime::new()?);
    let mut handlers = vec![];
    println!("{}", 1);
    let (client, tasks) = async_create_and_start_processor(
        rx,
        IndexerConfiguration {
            mq: ZMQConfiguration {
                zmq_url: "tcp://127.0.0.1:30001".to_string(),
                zmq_topic: vec![
                    "hashtx".to_string(),
                    "hashblock".to_string(),
                    "tx".to_string(),
                    "block".to_string(),
                ],
            },
            net: NetConfiguration {
                url: "http://localhost:18443".to_string(),
                username: "devnet".to_string(),
                password: "devnet".to_string(),
            },
            db_path: "./chaindata/bitcoind".to_string(),
            save_block_cache_count: 10,
            log_configuration: Default::default(),
        },
        rt,
    )
    .await;
    handlers.extend(tasks);
    let (tx, rx) = async_channel::unbounded();
    handlers.push(
        tokio::spawn(async move {
            let ctx = client.rx();
            loop {
                let data = ctx.recv().await;
                if let Err(e) = data {
                    println!("{}", 2);
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
                println!("{:?}", data);
                let transaction = data.unwrap();
                tx.send(transaction).await.expect("unreachable")
            }
        })
        .await
        .unwrap(),
    );

    handlers.push(tokio::spawn(async move {
        loop {
            let data = rx.recv().await;
            if let Err(e) = data {
                sleep(Duration::from_secs(1)).await;
                continue;
            }
            println!("{:?}", data);
            let event = data.unwrap();

            match event {
                ClientEvent::Transaction(tx) => {
                    let tx_id: TxIdType = tx.txid().into();
                    tracing::info!("tx_id: {:?}", tx_id);
                }
                ClientEvent::GetHeight => {}
                ClientEvent::TxDroped(tx_id) => {
                    tracing::info!("tx_id: {:?}", tx_id);
                }
                ClientEvent::TxConfirmed(tx_id) => {
                    tracing::info!("tx_id: {:?}", tx_id);
                }
            };
            // if let Err(e) = ctx_res {
            //     sleep(Duration::from_secs(1)).await;
            //     continue;
            // }
            // let ctx = ctx_res.unwrap();
            // if let Some(ctx) = ctx {
            //     client
            //         .update_delta(ctx.delta.clone())
            //         .await
            //         .expect("unreachable");
            // }
        }
    }));

    for h in handlers {
        h.await.unwrap();
    }

    // HttpServer::new(move || {
    //     App::new()
    //         .wrap(cors())
    //         .wrap(Logger::default())
    //         .configure(route)
    //         .default_service(web::to(|| HttpResponse::NotFound()))
    // })
    // .bind("0.0.0.0:8888")?
    // .shutdown_timeout(2)
    // .workers(1)
    // .run()
    // .await?;
    Ok(())
}

fn cors() -> Cors {
    Cors::default()
        .allow_any_origin()
        .allowed_methods(vec!["PUT", "DELETE", "POST", "GET", "OPTIONS"])
        .allowed_headers(vec![
            "Content-Type",
            "Authorization",
            "X-Requested-With",
            "Access-Control-Allow-Origin",
        ])
        .supports_credentials()
        .max_age(86400)
}

pub fn route(cfg: &mut web::ServiceConfig) {}
