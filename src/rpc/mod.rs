use std::pin::Pin;
use std::time::Instant;

use futures::Stream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, async_trait};

use crate::{MethodRegistry, RpcError};

/// Re-export the protobuf-generated types so downstream modules do not have to
/// reach into the `generated/` folder directly.
pub mod proto {
    #![allow(clippy::all, missing_docs)]
    pub mod waemu {
        pub mod rpc {
            pub mod v1 {
                include!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/generated/waemu.rs"
                ));
            }
        }
    }
}

use proto::waemu::rpc::v1::{
    ErrorDetail, InvokeRequest, InvokeResponse,
    invoke_plane_server::{InvokePlane, InvokePlaneServer},
};

const STREAM_BUFFER: usize = 16;

/// gRPC service surface that executes method invocations against the local [`MethodRegistry`].
#[derive(Clone)]
pub struct InvokeService {
    registry: MethodRegistry,
}

impl InvokeService {
    /// Creates an [`InvokeService`] bound to the provided registry.
    pub fn new(registry: MethodRegistry) -> Self {
        Self { registry }
    }

    /// Convenience helper to wrap the service in a tonic server.
    pub fn into_server(self) -> InvokePlaneServer<Self> {
        InvokePlaneServer::new(self)
    }
}

#[async_trait]
impl InvokePlane for InvokeService {
    async fn invoke(
        &self,
        request: Request<InvokeRequest>,
    ) -> Result<Response<InvokeResponse>, Status> {
        let response = execute_request(&self.registry, request.into_inner()).await?;
        Ok(Response::new(response))
    }

    type InvokeStreamStream = ReceiverStream<Result<InvokeResponse, Status>>;

    async fn invoke_stream(
        &self,
        request: Request<tonic::Streaming<InvokeRequest>>,
    ) -> Result<Response<<Self as InvokePlane>::InvokeStreamStream>, Status> {
        let mut inbound = request.into_inner();
        let registry = self.registry.clone();
        let (tx, rx) = mpsc::channel(STREAM_BUFFER);

        tokio::spawn(async move {
            while let Some(next) = inbound.message().await.transpose() {
                match next {
                    Ok(msg) => {
                        let result = execute_request(&registry, msg).await;
                        if tx.send(result).await.is_err() {
                            break;
                        }
                    }
                    Err(status) => {
                        let _ = tx.send(Err(status));
                        break;
                    }
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

async fn execute_request(
    registry: &MethodRegistry,
    request: InvokeRequest,
) -> Result<InvokeResponse, Status> {
    let InvokeRequest {
        peer_id,
        method,
        payload,
        correlation_id,
        timeout_ms: _,
    } = request;

    if method.trim().is_empty() {
        return Err(Status::invalid_argument("method must not be empty"));
    }

    let started = Instant::now();
    let result = registry.invoke(&method, payload);
    let elapsed_ms = started.elapsed().as_millis() as u64;

    let (success, result_payload, error) = match result {
        Ok(bytes) => (true, bytes, None),
        Err(err) => (false, Vec::new(), Some(to_error_detail(err))),
    };

    Ok(InvokeResponse {
        peer_id,
        method,
        result: result_payload,
        success,
        error,
        correlation_id,
        elapsed_ms,
    })
}

fn to_error_detail(err: RpcError) -> ErrorDetail {
    let code = err.code().to_string();
    let message = match &err {
        RpcError::Config(msg) => msg.clone(),
        RpcError::MethodNotFound(method) => format!("method {method} not registered"),
        RpcError::Transport(msg) => msg.clone(),
        RpcError::Internal(msg) => msg.clone(),
    };

    ErrorDetail {
        code,
        message,
        details: Vec::new(),
    }
}

/// Convenience alias for boxed streaming responses.
pub type InvokeStream =
    Pin<Box<dyn Stream<Item = Result<InvokeResponse, Status>> + Send + Sync + 'static>>;
