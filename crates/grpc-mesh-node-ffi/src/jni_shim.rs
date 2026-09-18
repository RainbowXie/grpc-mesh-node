//! JNI shim for the Java/Android `io.mesh.node` binding.
//!
//! Design rules (D8/D9): the shim only adapts the stable C ABI — every
//! operation goes through the same `mesh_node_*` entry points a C host
//! would call — and never touches Node Core internals. The shim owns:
//!
//! * the `JavaVM` (saved in `JNI_OnLoad`),
//! * per-registration `GlobalRef`s to Java handler objects (released on
//!   replace/unregister, and on free after the Java layer stopped the
//!   node successfully),
//! * thread attachment discipline: callbacks from Rust worker threads
//!   attach temporarily and detach afterwards; threads already attached
//!   by the JVM are reused and never detached.

use std::collections::HashMap;
use std::ffi::{CStr, CString, c_long, c_void};
use std::ptr;
use std::sync::{LazyLock, Mutex};

use jni::JNIEnv;
use jni::objects::{GlobalRef, JObject, JString, JThrowable, JValue};
use jni::sys::{
    JNI_VERSION_1_6, JavaVM as RawJavaVM, jint, jlong, jobject, jstring,
};

use crate::{
    MESH_NODE_BUSINESS_INTERNAL, MESH_NODE_ERR_INVALID_STATE, MESH_NODE_ERR_NOT_FOUND,
    MESH_NODE_ERR_SHUTDOWN_TIMEOUT, MESH_NODE_OK, MeshNodeHandle, MeshNodeRequest,
    mesh_node_last_error,
};

/// Raw pointer wrapper that is safe to move between threads: the JavaVM
/// handle is process-global and valid after JNI_OnLoad.
#[repr(transparent)]
struct SendVm(*mut RawJavaVM);
unsafe impl Send for SendVm {}

static VM: LazyLock<Mutex<Option<SendVm>>> = LazyLock::new(|| Mutex::new(None));

/// Handler registrations alive per node, stored as addresses of owned
/// `Box<JniHandler>` allocations. Each box has a single owner (the
/// registration it backs); the map exists so free() can release
/// GlobalRefs that were never unregistered. Addresses (not pointers) keep
/// the map Send.
static HANDLERS: LazyLock<Mutex<HashMap<MeshNodeHandle, Vec<usize>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

struct JniHandler {
    handler: GlobalRef,
}

// SAFETY: the raw pointer is only dereferenced from the trampoline while
// the registration exists, and GlobalRef itself is Send + Sync.
unsafe impl Send for JniHandler {}

const NATIVE_INVOKE_SIG: &str = "([BLjava/lang/String;Ljava/lang/String;J)[B";
const CLASS: &str = "io/mesh/node/GrpcMeshNode";

/// Called by the JVM when the library is loaded; saves the JavaVM.
/// cbindgen:ignore
#[unsafe(no_mangle)]
pub unsafe extern "C" fn JNI_OnLoad(vm: *mut RawJavaVM, _reserved: *mut c_void) -> jint {
    *VM.lock().expect("vm lock poisoned") = Some(SendVm(vm));
    JNI_VERSION_1_6
}

fn with_vm<R>(f: impl FnOnce(&jni::JavaVM) -> Result<R, String>) -> Result<R, String> {
    let vm_ptr = {
        let guard = VM.lock().expect("vm lock poisoned");
        guard
            .as_ref()
            .map(|send| send.0)
            .ok_or_else(|| "JavaVM not initialized (JNI_OnLoad not called)".to_string())?
    };
    // SAFETY: the raw pointer comes from JNI_OnLoad and remains valid for
    // the process lifetime; reconstructing the wrapper is cheap.
    let vm = unsafe { jni::JavaVM::from_raw(vm_ptr) }.map_err(|e| e.to_string())?;
    f(&vm)
}

fn jstring_to_string(env: &mut JNIEnv, s: JString<'_>) -> Result<String, String> {
    env.get_string(&s)
        .map(|java| java.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}

fn last_error_message() -> String {
    unsafe { CStr::from_ptr(mesh_node_last_error()).to_string_lossy().into_owned() }
}

fn throw(env: &mut JNIEnv, class: &str, message: &str) {
    let _ = env.throw_new(class, message);
}

fn map_status_exception(status: i32) -> &'static str {
    match status {
        MESH_NODE_ERR_INVALID_STATE | MESH_NODE_ERR_NOT_FOUND => {
            "io/mesh/node/NodeStateException"
        }
        MESH_NODE_ERR_SHUTDOWN_TIMEOUT => "io/mesh/node/ShutdownTimeoutException",
        _ => "io/mesh/node/MeshNodeException",
    }
}

// ---------------------------------------------------------------------------
// JNI entry points (io.mesh.node.GrpcMeshNode)
// ---------------------------------------------------------------------------

/// Returns the runtime ABI version (delegates to the C ABI).
/// cbindgen:ignore
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_io_mesh_node_GrpcMeshNode_nativeAbiVersion(
    _env: JNIEnv,
    _class: jni::objects::JClass<'_>,
) -> jint {
    crate::mesh_node_abi_version() as jint
}

/// Creates a node; returns the handle or throws NodeConfigException.
/// cbindgen:ignore
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_io_mesh_node_GrpcMeshNode_nativeCreate(
    mut env: JNIEnv,
    _class: jni::objects::JClass<'_>,
    config_json: JString<'_>,
) -> jlong {
    let json = match jstring_to_string(&mut env, config_json) {
        Ok(json) => json,
        Err(err) => {
            throw(&mut env, CLASS, &format!("reading config: {err}"));
            return 0;
        }
    };
    let Ok(c_json) = CString::new(json) else {
        throw(&mut env, CLASS, "config contains interior NUL");
        return 0;
    };
    let handle = unsafe { crate::mesh_node_new(c_json.as_ptr()) };
    if handle == 0 {
        throw(
            &mut env,
            "io/mesh/node/NodeConfigException",
            &last_error_message(),
        );
        return 0;
    }
    handle as jlong
}

/// Registers a Java handler; replacements release the previous GlobalRef.
/// cbindgen:ignore
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_io_mesh_node_GrpcMeshNode_nativeRegisterMethod(
    mut env: JNIEnv,
    _this: jni::objects::JObject<'_>,
    handle: jlong,
    method: JString<'_>,
    handler: jobject,
) {
    let method_name = match jstring_to_string(&mut env, method) {
        Ok(name) => name,
        Err(err) => {
            throw(&mut env, CLASS, &format!("reading method name: {err}"));
            return;
        }
    };
    // SAFETY: jobject is a valid local reference passed in by the JVM.
    let handler_obj = unsafe { jni::objects::JObject::from_raw(handler) };
    let global = match env.new_global_ref(&handler_obj) {
        Ok(global) => global,
        Err(err) => {
            throw(&mut env, CLASS, &format!("global ref for handler: {err}"));
            return;
        }
    };
    let user_data = Box::into_raw(Box::new(JniHandler { handler: global })) as *mut c_void;

    let Ok(c_method) = CString::new(method_name) else {
        drop(unsafe { Box::from_raw(user_data as *mut JniHandler) });
        throw(&mut env, CLASS, "method name contains interior NUL");
        return;
    };
    let mut replaced: *mut c_void = ptr::null_mut();
    let status = unsafe {
        crate::mesh_node_register_method(
            handle as MeshNodeHandle,
            c_method.as_ptr(),
            jni_trampoline,
            user_data,
            &mut replaced,
        )
    };
    if status != MESH_NODE_OK {
        drop(unsafe { Box::from_raw(user_data as *mut JniHandler) });
        throw(&mut env, map_status_exception(status), &last_error_message());
        return;
    }
    // A replacement hands the previous registration back for release; its
    // tracked entry must go too, or free() would drop the same allocation
    // a second time.
    let mut handlers = HANDLERS.lock().expect("handlers lock poisoned");
    if !replaced.is_null() {
        drop(unsafe { Box::from_raw(replaced as *mut JniHandler) });
        if let Some(list) = handlers.get_mut(&(handle as MeshNodeHandle)) {
            list.retain(|tracked| *tracked != replaced as usize);
        }
    }
    handlers
        .entry(handle as MeshNodeHandle)
        .or_default()
        .push(user_data as usize);
}

/// Unregisters a method, releasing its GlobalRef.
/// cbindgen:ignore
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_io_mesh_node_GrpcMeshNode_nativeUnregisterMethod(
    mut env: JNIEnv,
    _this: jni::objects::JObject<'_>,
    handle: jlong,
    method: JString<'_>,
) {
    let method_name = match jstring_to_string(&mut env, method) {
        Ok(name) => name,
        Err(err) => {
            throw(&mut env, CLASS, &format!("reading method name: {err}"));
            return;
        }
    };
    let Ok(c_method) = CString::new(method_name) else {
        throw(&mut env, CLASS, "method name contains interior NUL");
        return;
    };
    let mut removed: *mut c_void = ptr::null_mut();
    let status = unsafe {
        crate::mesh_node_unregister_method(
            handle as MeshNodeHandle,
            c_method.as_ptr(),
            &mut removed,
        )
    };
    if status != MESH_NODE_OK {
        throw(&mut env, map_status_exception(status), &last_error_message());
        return;
    }
    if !removed.is_null() {
        drop(unsafe { Box::from_raw(removed as *mut JniHandler) });
        if let Ok(mut handlers) = HANDLERS.lock() {
            if let Some(list) = handlers.get_mut(&(handle as MeshNodeHandle)) {
                list.retain(|tracked| *tracked != removed as usize);
            }
        }
    }
}

/// cbindgen:ignore
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_io_mesh_node_GrpcMeshNode_nativeStart(
    mut env: JNIEnv,
    _this: jni::objects::JObject<'_>,
    handle: jlong,
) {
    let status = unsafe { crate::mesh_node_start(handle as MeshNodeHandle) };
    if status != MESH_NODE_OK {
        throw(&mut env, map_status_exception(status), &last_error_message());
    }
}

/// cbindgen:ignore
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_io_mesh_node_GrpcMeshNode_nativeStop(
    mut env: JNIEnv,
    _this: jni::objects::JObject<'_>,
    handle: jlong,
    timeout_ms: jlong,
) {
    let timeout = u32::try_from(timeout_ms.max(0)).unwrap_or(u32::MAX);
    let status = unsafe { crate::mesh_node_stop(handle as MeshNodeHandle, timeout) };
    if status != MESH_NODE_OK {
        throw(&mut env, map_status_exception(status), &last_error_message());
    }
}

/// Frees the node and releases every remaining handler GlobalRef. The Java
/// layer guarantees this is only reached after a successful stop.
/// cbindgen:ignore
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_io_mesh_node_GrpcMeshNode_nativeFree(
    _env: JNIEnv,
    _class: jni::objects::JClass<'_>,
    handle: jlong,
) {
    let remaining = HANDLERS
        .lock()
        .expect("handlers lock poisoned")
        .remove(&(handle as MeshNodeHandle))
        .unwrap_or_default();
    for tracked in remaining {
        drop(unsafe { Box::from_raw(tracked as *mut JniHandler) });
    }
    unsafe { crate::mesh_node_free(handle as MeshNodeHandle) };
}

/// Returns the node's lifecycle state (1..5) or 0 for an invalid handle.
/// cbindgen:ignore
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_io_mesh_node_GrpcMeshNode_nativeState(
    _env: JNIEnv,
    _class: jni::objects::JClass<'_>,
    handle: jlong,
) -> jint {
    unsafe { crate::mesh_node_state(handle as MeshNodeHandle) }
}

/// cbindgen:ignore
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_io_mesh_node_GrpcMeshNode_nativeLastError(
    mut env: JNIEnv,
    _class: jni::objects::JClass<'_>,
) -> jstring {
    match env.new_string(last_error_message()) {
        Ok(string) => string.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}

// ---------------------------------------------------------------------------
// Callback trampoline
// ---------------------------------------------------------------------------

unsafe extern "C-unwind" fn jni_trampoline(
    request: *const MeshNodeRequest,
    user_data: *mut c_void,
) -> u64 {
    let handler = unsafe { &*(user_data as *const JniHandler) };
    match run_java_handler(handler, unsafe { &*request }) {
        Ok(bytes) => unsafe { crate::mesh_node_response_ok(bytes.as_ptr(), bytes.len()) },
        Err(message) => {
            let Ok(msg) = CString::new(message) else {
                return unsafe {
                    crate::mesh_node_response_error(
                        MESH_NODE_BUSINESS_INTERNAL,
                        b"java handler error\0".as_ptr().cast(),
                    )
                };
            };
            unsafe {
                crate::mesh_node_response_error(MESH_NODE_BUSINESS_INTERNAL, msg.as_ptr())
            }
        }
    }
}

fn run_java_handler(handler: &JniHandler, request: &MeshNodeRequest) -> Result<Vec<u8>, String> {
    with_vm(|vm| {
        // D9: threads already attached by the JVM are reused and never
        // detached; unattached Rust workers attach only for this callback.
        match vm.get_env() {
            Ok(mut env) => dispatch(&mut env, handler, request),
            Err(_) => {
                let Ok(mut guard) = vm.attach_current_thread() else {
                    return Err("attaching JVM thread failed".into());
                };
                let result = dispatch(&mut guard, handler, request);
                result // guard drop detaches this thread
            }
        }
    })
}

/// Dispatch error: either a JNI-layer failure or an application-level
/// message (handler threw, malformed response, ...). Implements
/// `From<jni::errors::Error>` so it fits `JNIEnv::with_local_frame`.
enum DispatchError {
    Jni(jni::errors::Error),
    Msg(String),
}

impl From<jni::errors::Error> for DispatchError {
    fn from(err: jni::errors::Error) -> Self {
        Self::Jni(err)
    }
}

impl DispatchError {
    fn message(self) -> String {
        match self {
            Self::Jni(err) => err.to_string(),
            Self::Msg(msg) => msg,
        }
    }
}

unsafe fn request_payload(request: &MeshNodeRequest) -> Result<&[u8], &'static str> {
    // The C ABI deliberately represents an empty payload as NULL + 0.
    // Rust slices require a non-null aligned pointer even at length zero,
    // so only construct a borrowed slice when bytes are actually present.
    if request.payload_len == 0 {
        Ok(&[])
    } else if request.payload.is_null() {
        Err("request payload is null with non-zero length")
    } else {
        Ok(unsafe { std::slice::from_raw_parts(request.payload, request.payload_len) })
    }
}

fn dispatch(
    env: &mut JNIEnv,
    handler: &JniHandler,
    request: &MeshNodeRequest,
) -> Result<Vec<u8>, String> {
    env.with_local_frame(32, |env| {
        let payload = unsafe { request_payload(request) }
            .map_err(|message| DispatchError::Msg(message.to_owned()))?;
        let j_payload = env
            .byte_array_from_slice(payload)
            .map_err(|e| DispatchError::Msg(format!("payload array: {e}")))?;
        let method = unsafe { CStr::from_ptr(request.method) }
            .to_string_lossy()
            .into_owned();
        let j_method = env
            .new_string(method)
            .map_err(|e| DispatchError::Msg(format!("method string: {e}")))?;
        let correlation = unsafe { CStr::from_ptr(request.correlation_id) }
            .to_string_lossy()
            .into_owned();
        let j_correlation = env
            .new_string(correlation)
            .map_err(|e| DispatchError::Msg(format!("correlation string: {e}")))?;
        let timeout = c_long::from(request.timeout_ms);

        let result = env.call_method(
            handler.handler.as_obj(),
            "nativeInvoke",
            NATIVE_INVOKE_SIG,
            &[
                JValue::Object(&j_payload),
                JValue::Object(&j_method),
                JValue::Object(&j_correlation),
                JValue::Long(timeout),
            ],
        );

        // Java exceptions are extracted, cleared and converted to a
        // structured error; the JVM and the Rust runtime keep working.
        // Grab the throwable local reference BEFORE clearing: calling
        // methods with a pending exception is not allowed on ART, so the
        // description is built only after the exception is cleared.
        if env.exception_check().unwrap_or(false) {
            let throwable = env.exception_occurred().ok();
            if env.exception_clear().is_err() {
                return Err(DispatchError::Msg("clearing java exception failed".into()));
            }
            let description = throwable
                .as_ref()
                .and_then(|throwable| describe_throwable(env, throwable))
                .unwrap_or_else(|| "java exception".to_string());
            return Err(DispatchError::Msg(format!("java handler threw: {description}")));
        }

        let output =
            result.map_err(|e| DispatchError::Msg(format!("invoking handler: {e}")))?;
        let object: JObject<'_> = output
            .l()
            .map_err(|_| DispatchError::Msg("handler returned non-object".into()))?;
        let array: jni::objects::JByteArray<'_> =
            unsafe { jni::objects::JByteArray::from_raw(object.into_raw()) };
        let len = env
            .get_array_length(&array)
            .map_err(|e| DispatchError::Msg(format!("response length: {e}")))?;
        if len < 0 {
            return Err(DispatchError::Msg("negative response length".into()));
        }
        let mut buffer = vec![0u8; len as usize];
        // SAFETY: jbyte is i8; a u8 slice of equal length has the same layout.
        let raw = unsafe {
            std::slice::from_raw_parts_mut(buffer.as_mut_ptr() as *mut i8, buffer.len())
        };
        env.get_byte_array_region(&array, 0, raw)
            .map_err(|e| DispatchError::Msg(format!("response bytes: {e}")))?;
        Ok(buffer)
    })
    .map_err(|err: DispatchError| err.message())
}

#[cfg(test)]
mod tests {
    use super::request_payload;
    use crate::MeshNodeRequest;
    use std::ptr;

    fn request(payload: *const u8, payload_len: usize) -> MeshNodeRequest {
        MeshNodeRequest {
            payload,
            payload_len,
            method: ptr::null(),
            correlation_id: ptr::null(),
            timeout_ms: 0,
        }
    }

    #[test]
    fn null_pointer_with_zero_length_is_an_empty_payload() {
        let request = request(ptr::null(), 0);
        let payload = unsafe { request_payload(&request) }.expect("empty payload is valid");
        assert!(payload.is_empty());
    }

    #[test]
    fn null_pointer_with_nonzero_length_is_rejected() {
        let request = request(ptr::null(), 1);
        assert_eq!(
            unsafe { request_payload(&request) }.expect_err("invalid payload must fail"),
            "request payload is null with non-zero length"
        );
    }

    #[test]
    fn nonempty_payload_is_borrowed_without_truncation() {
        let bytes = b"a\0b";
        let request = request(bytes.as_ptr(), bytes.len());
        assert_eq!(
            unsafe { request_payload(&request) }.expect("payload is valid"),
            bytes
        );
    }
}

fn describe_throwable(env: &mut JNIEnv, throwable: &JThrowable<'_>) -> Option<String> {
    let class_obj: JObject<'_> = env
        .call_method(throwable, "getClass", "()Ljava/lang/Class;", &[])
        .ok()?
        .l()
        .ok()?;
    let name_obj: JObject<'_> = env
        .call_method(&class_obj, "getName", "()Ljava/lang/String;", &[])
        .ok()?
        .l()
        .ok()?;
    let name_string: JString<'_> = name_obj.into();
    let class_name = env
        .get_string(&name_string)
        .ok()?
        .to_string_lossy()
        .into_owned();

    let message_obj: Option<JObject<'_>> = env
        .call_method(throwable, "getMessage", "()Ljava/lang/String;", &[])
        .ok()
        .and_then(|value| value.l().ok());
    let message = message_obj
        .map(JString::from)
        .and_then(|string| {
            env.get_string(&string)
                .ok()
                .map(|s| s.to_string_lossy().into_owned())
        })
        .unwrap_or_default();

    Some(if message.is_empty() { class_name } else { format!("{class_name}: {message}") })
}
