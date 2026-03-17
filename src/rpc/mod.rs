use std::pin::Pin;
use std::time::Instant;

use futures::Stream;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, async_trait};

use crate::{MethodRegistry, RpcError};

/// Protocol buffer definitions for the InvokePlaneService gRPC service.
///
/// This module re-exports the generated protobuf types for the gRPC-Mesh RPC protocol.
/// It includes the `InvokePlaneService` service definition and associated request/response types.
///
/// # Generated Types
///
/// - `InvokeRequest`: Request message for unary invocations
/// - `InvokeResponse`: Response message containing results or errors
/// - `ErrorDetail`: Structured error information
/// - `InvokePlaneService`: Service trait for implementing the RPC handler
/// - `InvokePlaneServiceServer`: Server implementation wrapper
///
/// These types are used by both the `InvokeService` implementation and client code
/// that needs to interact with the mesh protocol.
pub mod proto {
    #![allow(clippy::all, missing_docs)]
    pub mod grpc_mesh {
        pub mod rpc {
            pub mod v1 {
                include!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/src/generated/grpc_mesh.rs"
                ));
            }
        }
    }
}

use proto::grpc_mesh::rpc::v1::{
    ErrorDetail, InvokeRequest, InvokeResponse, InvokeStreamRequest, InvokeStreamResponse,
    invoke_plane_service_server::{InvokePlaneService, InvokePlaneServiceServer},
};

const STREAM_BUFFER: usize = 16;

/// gRPC service implementation that dispatches RPC invocations to local method handlers.
///
/// `InvokeService` implements the `InvokePlaneService` gRPC service by routing incoming
/// requests to the appropriate handlers registered in a [`MethodRegistry`]. It provides:
/// - Unary RPC support via `Invoke`
/// - Bidirectional streaming via `InvokeStream`
/// - Automatic error handling and response marshaling
/// - Request timing and observability
///
/// This service is designed to be used with tonic's server builder and runs over
/// the Yamux-multiplexed tunnel connection to the gRPC-Mesh server.
///
/// # Example
///
/// ```no_run
/// use grpc_mesh_node::{MethodRegistry, rpc::InvokeService};
/// use std::sync::Arc;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Create registry and register methods
/// let registry = MethodRegistry::default();
/// registry.register("calculator.add", Arc::new(|payload| {
///     // Handler implementation...
///     Ok(vec![42])
/// }));
///
/// // Create service
/// let service = InvokeService::new(registry);
///
/// // Convert to tonic server and serve
/// let server = service.into_server();
/// // Use with tonic::transport::Server::builder()...
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct InvokeService {
    registry: MethodRegistry,
}

impl InvokeService {
    /// Creates a new InvokeService with the given method registry.
    ///
    /// The service will route all incoming RPC requests to handlers registered
    /// in the provided registry. The registry can be shared and updated even
    /// after the service is created, as it uses interior mutability.
    ///
    /// # Parameters
    ///
    /// - `registry`: Method registry containing registered RPC handlers
    ///
    /// # Example
    ///
    /// ```
    /// use grpc_mesh_node::{MethodRegistry, rpc::InvokeService};
    /// use std::sync::Arc;
    ///
    /// let registry = MethodRegistry::default();
    /// registry.register("echo", Arc::new(|payload| Ok(payload)));
    ///
    /// let service = InvokeService::new(registry);
    /// ```
    pub fn new(registry: MethodRegistry) -> Self {
        Self { registry }
    }

    /// Converts this service into a tonic server implementation.
    ///
    /// This is a convenience method that wraps the service in tonic's
    /// `InvokePlaneServiceServer` wrapper, making it ready to be added to a
    /// `tonic::transport::Server`.
    ///
    /// # Returns
    ///
    /// A tonic server instance that can be added to a server builder.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use grpc_mesh_node::{MethodRegistry, rpc::InvokeService};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let registry = MethodRegistry::default();
    /// let service = InvokeService::new(registry).into_server();
    ///
    /// tonic::transport::Server::builder()
    ///     .add_service(service)
    ///     .serve("0.0.0.0:50051".parse()?)
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn into_server(self) -> InvokePlaneServiceServer<Self> {
        InvokePlaneServiceServer::new(self)
    }
}

#[async_trait]
impl InvokePlaneService for InvokeService {
    /// Handles unary RPC invocations.
    ///
    /// This method:
    /// 1. Extracts the InvokeRequest from the gRPC request
    /// 2. Routes it to the appropriate handler in the registry
    /// 3. Captures timing information
    /// 4. Returns the result or error wrapped in an InvokeResponse
    ///
    /// Errors from the handler are captured and returned in the response's
    /// error field rather than as gRPC status errors.
    async fn invoke(
        &self,
        request: Request<InvokeRequest>,
    ) -> Result<Response<InvokeResponse>, Status> {
        let response = execute_request(&self.registry, request.into_inner()).await?;
        Ok(Response::new(response))
    }

    type InvokeStreamStream = ReceiverStream<Result<InvokeStreamResponse, Status>>;

    /// Handles bidirectional streaming RPC invocations.
    ///
    /// This method creates a bidirectional stream where:
    /// - The client can send multiple InvokeStreamRequest messages
    /// - Each request is processed independently and its response is sent back
    /// - The stream remains open until the client closes it or an error occurs
    ///
    /// Each request in the stream is handled by spawning a task that processes
    /// requests from the inbound stream and sends responses to the outbound stream.
    async fn invoke_stream(
        &self,
        request: Request<tonic::Streaming<InvokeStreamRequest>>,
    ) -> Result<Response<<Self as InvokePlaneService>::InvokeStreamStream>, Status> {
        let mut inbound = request.into_inner();
        let registry = self.registry.clone();
        let (tx, rx) = mpsc::channel(STREAM_BUFFER);

        tokio::spawn(async move {
            while let Some(next) = inbound.message().await.transpose() {
                match next {
                    Ok(req) => {
                        // Convert InvokeStreamRequest to InvokeRequest
                        let invoke_req = InvokeRequest {
                            peer_id: req.peer_id,
                            method: req.method,
                            payload: req.payload,
                            correlation_id: req.correlation_id,
                            timeout_ms: req.timeout_ms,
                        };

                        let result = execute_request(&registry, invoke_req).await;

                        // Convert InvokeResponse to InvokeStreamResponse
                        let stream_resp = match result {
                            Ok(resp) => Ok(InvokeStreamResponse {
                                peer_id: resp.peer_id,
                                method: resp.method,
                                result: resp.result,
                                success: resp.success,
                                error: resp.error,
                                correlation_id: resp.correlation_id,
                                elapsed_ms: resp.elapsed_ms,
                            }),
                            Err(e) => Err(e),
                        };

                        if tx.send(stream_resp).await.is_err() {
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

/// Executes a single RPC request against the method registry.
///
/// This function:
/// 1. Validates the request (method name must not be empty)
/// 2. Looks up and invokes the handler in the registry
/// 3. Measures execution time
/// 4. Constructs an InvokeResponse with the result or error
///
/// # Parameters
///
/// - `registry`: Method registry to look up handlers
/// - `request`: InvokeRequest containing method name and payload
///
/// # Returns
///
/// - `Ok(InvokeResponse)`: Response with result or error details
/// - `Err(Status)`: gRPC status error for invalid requests
///
/// # Errors
///
/// Returns a gRPC `INVALID_ARGUMENT` status if the method name is empty.
/// Handler errors are captured in the response's error field.
async fn execute_request(
    registry: &MethodRegistry,
    request: InvokeRequest,
) -> Result<InvokeResponse, Status> {
    let InvokeRequest {
        peer_id,
        method,
        payload,
        correlation_id,
        timeout_ms,
    } = request;

    if method.trim().is_empty() {
        return Err(Status::invalid_argument("method must not be empty"));
    }

    let started = Instant::now();
    let result = registry.invoke_request(crate::RpcRequest {
        method: method.clone(),
        payload,
        timeout: std::time::Duration::from_millis(timeout_ms as u64),
        correlation_id: correlation_id.clone(),
    });
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

/// Converts an RpcError into a protobuf ErrorDetail message.
///
/// This function maps internal RpcError types to the wire format used in
/// gRPC responses, including error codes and human-readable messages.
///
/// # Parameters
///
/// - `err`: The RpcError to convert
///
/// # Returns
///
/// An ErrorDetail message ready to be included in an InvokeResponse.
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

/// Convenience type alias for boxed streaming responses.
///
/// This type represents a dynamic, pinned stream of InvokeResponse messages
/// suitable for use in async contexts. It's primarily used for advanced
/// streaming scenarios where the concrete stream type needs to be erased.
///
/// Most users should use the `InvokeStreamStream` type from the `InvokePlaneService`
/// trait implementation instead.
pub type InvokeStream =
    Pin<Box<dyn Stream<Item = Result<InvokeStreamResponse, Status>> + Send + Sync + 'static>>;
