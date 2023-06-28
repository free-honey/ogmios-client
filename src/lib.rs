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
    async fn submit_tx(&self, tx: &[u8]) -> Result<OgmiosResponse<SubmitSuccess>>;
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
    SubmitTx {
        submit: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AdditionalUTxO {
    transaction_id: String,
    index: u64,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct OgmiosResponse<T> {
    #[serde(rename = "type")]
    message_type: String,
    version: String,
    #[serde(rename = "servicename")]
    service_name: String,
    #[serde(rename = "methodname")]
    method_name: Option<String>,
    result: Option<T>,
    fault: Option<serde_json::Value>,
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

    pub fn fault(&self) -> Option<&serde_json::Value> {
        self.fault.as_ref()
    }

    pub fn reflection(&self) -> Option<&serde_json::Value> {
        self.reflection.as_ref()
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct EvaluationResult(serde_json::Value);

impl EvaluationResult {
    pub fn value(&self) -> &serde_json::Value {
        &self.0
    }
}

#[derive(Deserialize, Serialize, Debug)]
// #[serde()]
pub struct SubmitSuccess {
    #[serde(rename = "SubmitSuccess")]
    inner: SubmitSuccessInner,
}

#[derive(Deserialize, Serialize, Debug)]
struct SubmitSuccessInner {
    #[serde(rename = "txId")]
    tx_id: String,
}

impl SubmitSuccess {
    pub fn new(tx_id: String) -> Self {
        SubmitSuccess {
            inner: SubmitSuccessInner { tx_id },
        }
    }
    pub fn tx_id(&self) -> &str {
        &self.inner.tx_id
    }
}

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

    async fn submit_tx(&self, tx: &[u8]) -> Result<OgmiosResponse<SubmitSuccess>> {
        let tx_hex = hex::encode(tx);

        let msg = OgmiosMessage {
            message_type: "jsonwsp/request".to_string(),
            version: "1.0".to_string(),
            service_name: "ogmios".to_string(),
            method_name: "SubmitTx".to_string(),
            args: MessageArgs::SubmitTx {
                submit: tx_hex.to_string(),
            },
        };
        let msg_str = serde_json::to_string(&msg).unwrap();
        let message = Message::Text(msg_str);
        let resp = self.message(message).await?;
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_correct() {
        let res = OgmiosResponse {
            message_type: "jsonwsp/response".to_string(),
            version: "1.0".to_string(),
            service_name: "ogmios".to_string(),
            method_name: Some("SubmitTx".to_string()),
            result: Some(SubmitSuccess::new(
                "b8a4628216237d47bb5bb095e79c9f91ccf043d15f55e87bf9df5a0d920022c2".to_string(),
            )),
            fault: None,
            reflection: None,
        };

        let expected = "{\"type\":\"jsonwsp/response\",\"version\":\"1.0\",\"servicename\":\"ogmios\",\"methodname\":\"SubmitTx\",\"result\":{\"SubmitSuccess\":{\"txId\":\"b8a4628216237d47bb5bb095e79c9f91ccf043d15f55e87bf9df5a0d920022c2\"}},\"fault\":null,\"reflection\":null}";
        let actual = serde_json::to_string(&res).unwrap();
        assert_eq!(expected, actual);
    }
}
