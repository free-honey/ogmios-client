use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tungstenite::{connect, Message};
use url::{ParseError, Url};

pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Tungstenite Error: {0}")]
    Tungstenite(#[from] tungstenite::Error),
    #[error("Url Error: {0}")]
    Url(#[from] ParseError),
    #[error("Serde Error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

#[async_trait]
pub trait OgmiosLocalTxSubmission {
    async fn evaluate_tx(
        &self,
        tx: &[u8],
        additional_utxo_set: Vec<AdditionalUTxO>,
    ) -> Result<OgmiosResponse<EvaluationResult>>;
    async fn submit_tx(&self, tx: &[u8], additional_utxo_set: Vec<AdditionalUTxO>) -> Result<()>;
}

pub struct OgmiosClient {
    ip: String,
    port: String,
}

impl OgmiosClient {
    pub fn new(ip: String, port: String) -> Self {
        OgmiosClient { ip, port }
    }

    async fn message<T: DeserializeOwned>(&self, msg: Message) -> Result<T> {
        let addr = format!("ws://{}:{}", self.ip, self.port);
        let url = Url::parse(&addr)?;
        let (mut socket, _) = connect(url)?;
        socket.write_message(msg)?;
        let res = socket.read_message();
        // println!("res: {:?}", res);
        match res {
            Ok(msg) => {
                let obj = serde_json::from_str(&msg.to_string())?;
                Ok(obj)
            }
            Err(err) => Err(Error::Tungstenite(err)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct OgmiosMessage {
    #[serde(rename = "type")]
    message_type: String,
    version: String,
    #[serde(rename = "servicename")]
    service_name: String,
    #[serde(rename = "methodname")]
    method_name: String,
    args: MessageArgs,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum MessageArgs {
    EvaluateTx {
        evaluate: String,
        #[serde(rename = "additionalUtxoSet")]
        additional_utxo_set: Vec<AdditionalUTxO>,
    },
    // SubmitTx(String),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdditionalUTxO {
    transaction_id: String,
    index: u64,
}

// {
//     "type":"jsonwsp/response",
//     "version":"1.0",
//     "servicename":"ogmios",
//     "methodname":"EvaluateTx",
//     "result":{
//         "EvaluationResult":{
//             "spend:0":{
//                 "memory":2301,
//                 "steps":586656
//             }
//         }
//     },
//     "reflection":null
// }
#[derive(Deserialize, Debug)]
pub struct OgmiosResponse<T> {
    #[serde(rename = "type")]
    message_type: String,
    version: String,
    #[serde(rename = "servicename")]
    service_name: String,
    #[serde(rename = "methodname")]
    method_name: Option<String>,
    result: Option<T>,
    fault: Option<Fault>,
    reflection: Option<serde_json::Value>,
}

impl<T> OgmiosResponse<T> {
    pub fn message_type(&self) -> &str {
        &self.message_type
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    pub fn method_name(&self) -> Option<&str> {
        self.method_name.as_deref()
    }

    pub fn result(&self) -> Option<&T> {
        self.result.as_ref()
    }

    pub fn fault(&self) -> Option<&Fault> {
        self.fault.as_ref()
    }

    pub fn reflection(&self) -> Option<&serde_json::Value> {
        self.reflection.as_ref()
    }
}

#[derive(Deserialize, Debug)]
pub struct Fault {
    code: String,
    #[serde(rename = "string")]
    message: String,
}

impl Fault {
    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Deserialize, Debug)]
pub struct EvaluationResult(serde_json::Value);

#[async_trait]
impl OgmiosLocalTxSubmission for OgmiosClient {
    async fn evaluate_tx(
        &self,
        tx: &[u8],
        additional_utxo_set: Vec<AdditionalUTxO>,
    ) -> Result<OgmiosResponse<EvaluationResult>> {
        let tx_hex = hex::encode(tx);

        let msg = OgmiosMessage {
            message_type: "jsonwsp/request".to_string(),
            version: "1.0".to_string(),
            service_name: "ogmios".to_string(),
            method_name: "EvaluateTx".to_string(),
            args: MessageArgs::EvaluateTx {
                evaluate: tx_hex.to_string(),
                additional_utxo_set,
            },
        };
        let msg_str = serde_json::to_string(&msg).unwrap();
        let message = Message::Text(msg_str);
        let resp = self.message(message).await?;
        Ok(resp)
    }

    async fn submit_tx(&self, _tx: &[u8], _additional_utxo_set: Vec<AdditionalUTxO>) -> Result<()> {
        todo!()
    }
}
