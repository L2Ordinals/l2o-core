use std::net::SocketAddr;

use bytes::Buf;
use bytes::Bytes;
use chainhook_sdk::types::BitcoinBlockMetadata;
use chainhook_sdk::types::BlockIdentifier;
use chainhook_sdk::types::OrdinalOperation;
use chainhook_sdk::types::TransactionIdentifier;
use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Incoming as IncomingBody;
use hyper::header;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::Method;
use hyper::Request;
use hyper::Response;
use hyper::StatusCode;
use hyper_util::rt::TokioIo;
use l2o::inscription::L2OInscription;
use serde::Deserialize;
use serde::Serialize;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

type GenericError = Box<dyn std::error::Error + Send + Sync>;
type Result<T> = std::result::Result<T, GenericError>;
type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;
pub mod l2o;
pub mod proof;
pub mod store;
static INDEX: &[u8] = b"<a href=\"test.html\">test.html</a>";
static INTERNAL_SERVER_ERROR: &[u8] = b"Internal Server Error";
static NOTFOUND: &[u8] = b"Not Found";
static POST_DATA: &str = r#"{"original": "data"}"#;
static URL: &str = "http://127.0.0.1:1337/json_api";

async fn client_request_response() -> Result<Response<BoxBody>> {
    let req = Request::builder()
        .method(Method::POST)
        .uri(URL)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Full::new(Bytes::from(POST_DATA)))
        .unwrap();

    let host = req.uri().host().expect("uri has no host");
    let port = req.uri().port_u16().expect("uri has no port");
    let stream = TcpStream::connect(format!("{}:{}", host, port)).await?;
    let io = TokioIo::new(stream);

    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::task::spawn(async move {
        if let Err(err) = conn.await {
            println!("Connection error: {:?}", err);
        }
    });

    let web_res = sender.send_request(req).await?;

    let res_body = web_res.into_body().boxed();

    Ok(Response::new(res_body))
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BitcoinTransactionDataV2 {
    pub transaction_identifier: TransactionIdentifier,
    /// Transactions that are related to other transactions should include the
    /// transaction_identifier of these transactions in the metadata.
    pub metadata: BitcoinTransactionMetadataV2,
}

/// Extra data for Transaction
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BitcoinTransactionMetadataV2 {
    pub ordinal_operations: Vec<OrdinalOperation>,
    pub proof: Option<String>,
}

/// BitcoinBlock contain an array of Transactions that occurred at a particular
/// BlockIdentifier. A hard requirement for blocks returned by Rosetta
/// implementations is that they MUST be _inalterable_: once a client has
/// requested and received a block identified by a specific BlockIndentifier,
/// all future calls for that same BlockIdentifier must return the same block
/// contents.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct BitcoinBlockDataV2 {
    pub block_identifier: BlockIdentifier,
    pub parent_block_identifier: BlockIdentifier,
    /// The timestamp of the block in milliseconds since the Unix Epoch. The
    /// timestamp is stored in milliseconds because some blockchains produce
    /// blocks more often than once a second.
    pub timestamp: u32,
    pub transactions: Vec<BitcoinTransactionDataV2>,
    pub metadata: BitcoinBlockMetadata,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BitcoinChainhookOccurrencePayloadV2 {
    pub apply: Vec<BitcoinBlockDataV2>,
    pub rollback: Vec<BitcoinBlockDataV2>,
}

fn process_l2o_inscription(inscription: L2OInscription) -> Result<()> {
    match inscription {
        L2OInscription::Deploy(deploy) => {
            println!("Deploy: {:?}", deploy);
            Ok(())
        }
        L2OInscription::Block(block) => {
            println!("Block: {:?}", block);
            Ok(())
        }
    }
}
fn process_ordinal_ops(payload: &BitcoinChainhookOccurrencePayloadV2) -> Result<()> {
    for apply in payload.apply.iter() {
        for transaction in apply.transactions.iter() {
            for ordinal_operation in transaction.metadata.ordinal_operations.iter() {
                match ordinal_operation {
                    OrdinalOperation::InscriptionRevealed(revealed) => {
                        println!("{:?}", revealed);
                        if revealed.content_type == "text/plain;charset=utf-8" {
                            let decoded = hex::decode(&revealed.content_bytes[2..])?;
                            println!("{}", String::from_utf8(decoded.clone()).unwrap());
                            let inscription = serde_json::from_slice::<L2OInscription>(&decoded)?;
                            println!(
                                " in transaction {}",
                                transaction.transaction_identifier.hash
                            );
                            process_l2o_inscription(inscription)?;
                        }
                    }
                    OrdinalOperation::InscriptionTransferred(_) => {
                        println!("xfer")
                    }
                }
            }
        }
    }
    Ok(())
}
async fn api_post_response(req: Request<IncomingBody>) -> Result<Response<BoxBody>> {
    // Aggregate the body...
    let whole_body = req.collect().await?.aggregate();
    // Decode as JSON...
    let data =
        serde_json::from_reader::<_, BitcoinChainhookOccurrencePayloadV2>(whole_body.reader())?;
    process_ordinal_ops(&data)?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(full("ok"))?;
    Ok(response)
}

async fn api_get_response() -> Result<Response<BoxBody>> {
    let data = vec!["foo", "bar"];
    let res = match serde_json::to_string(&data) {
        Ok(json) => Response::builder()
            .header(header::CONTENT_TYPE, "application/json")
            .body(full(json))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(full(INTERNAL_SERVER_ERROR))
            .unwrap(),
    };
    Ok(res)
}

async fn response_examples(req: Request<IncomingBody>) -> Result<Response<BoxBody>> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => Ok(Response::new(full(INDEX))),
        (&Method::GET, "/test.html") => client_request_response().await,
        (&Method::POST, "/api/events") => api_post_response(req).await,
        (&Method::GET, "/json_api") => api_get_response().await,
        _ => {
            // Return 404 not found response.
            Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(full(NOTFOUND))
                .unwrap())
        }
    }
}

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody {
    Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub async fn listen() -> Result<()> {
    pretty_env_logger::init();

    let addr: SocketAddr = "127.0.0.1:1337".parse().unwrap();

    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on http://{}", addr);
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);

        tokio::task::spawn(async move {
            let service = service_fn(move |req| response_examples(req));

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                println!("Failed to serve connection: {:?}", err);
            }
        });
    }
}
