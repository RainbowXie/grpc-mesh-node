use flate2::read::GzDecoder;
use serde::Deserialize;
use std::collections::HashMap;
// This struct represents the metadata received in `frida::Message::Other`
// e.g. {"type":"send","payload":{...}}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FridaPayload {
    pub r#type: String, // "send"
    pub payload: HttpMetadata,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct HttpMetadata {
    pub protocol: String,
    pub stream_id: u32,
    pub r#type: String, // "RESPONSE"
    pub request_id: String,
    pub method: Option<String>,
    pub url: Option<String>,
    pub headers: HashMap<String, String>,
    pub raw_frame: Option<Vec<u8>>,
    // The 'body' is no longer here. It is passed as the `data` argument in `on_message`.
}

#[derive(Debug)]
pub struct HttpMessage {
    pub metadata: HttpMetadata,
    pub body: Option<Vec<u8>>,
}

#[allow(dead_code)]
pub fn g_unzip(data: &[u8]) {}
