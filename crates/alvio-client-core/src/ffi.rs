#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::event::ClientEvent;
use crate::room::ClientRoom;

use crate::state::ConnectionState;
use alvio_core::{RoomId, StreamKind, StreamLayer};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

/// Opaque client handle exposed across C FFI boundaries.
pub struct AlvioClientHandle {
    pub room: ClientRoom,
    pub runtime: Arc<Runtime>,
    pub event_rx: mpsc::UnboundedReceiver<ClientEvent>,
}

/// Create a new Alvio client handle. Returns NULL on failure.
#[no_mangle]
pub extern "C" fn alvio_client_create(version: *const c_char) -> *mut AlvioClientHandle {
    if version.is_null() {
        return std::ptr::null_mut();
    }

    let version_str = match unsafe { CStr::from_ptr(version) }.to_str() {
        Ok(s) => s,
        Err(_) => return std::ptr::null_mut(),
    };

    let runtime = match Runtime::new() {
        Ok(rt) => Arc::new(rt),
        Err(_) => return std::ptr::null_mut(),
    };

    let room = ClientRoom::new(version_str);
    let mut broadcast_rx = room.subscribe_events();
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    runtime.spawn(async move {
        while let Ok(event) = broadcast_rx.recv().await {
            if event_tx.send(event).is_err() {
                break;
            }
        }
    });

    Box::into_raw(Box::new(AlvioClientHandle {
        room,
        runtime,
        event_rx,
    }))
}

/// Free the client handle and its associated background runtime.
#[no_mangle]
pub extern "C" fn alvio_client_destroy(handle: *mut AlvioClientHandle) {
    if !handle.is_null() {
        let boxed = unsafe { Box::from_raw(handle) };
        boxed.runtime.block_on(async {
            boxed.room.disconnect().await;
        });
    }
}

/// Get current connection state as integer:
/// 0 = Disconnected, 1 = Connecting, 2 = Connected, 3 = InRoom, 4 = Reconnecting, 5 = Failed
#[no_mangle]
pub extern "C" fn alvio_client_state(handle: *mut AlvioClientHandle) -> i32 {
    if handle.is_null() {
        return -1;
    }
    let client = unsafe { &*handle };
    match client.room.state() {
        ConnectionState::Disconnected => 0,
        ConnectionState::Connecting => 1,
        ConnectionState::Connected => 2,
        ConnectionState::InRoom => 3,
        ConnectionState::Reconnecting => 4,
        ConnectionState::Failed => 5,
    }
}

/// Join a room by ID and peer display name. Returns 0 on success, -1 on failure.
#[no_mangle]
pub extern "C" fn alvio_client_join(
    handle: *mut AlvioClientHandle,
    room_id: *const c_char,
    peer_name: *const c_char,
) -> i32 {
    if handle.is_null() || room_id.is_null() || peer_name.is_null() {
        return -1;
    }

    let room_id_str = match unsafe { CStr::from_ptr(room_id) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let peer_name_str = match unsafe { CStr::from_ptr(peer_name) }.to_str() {
        Ok(s) => s,
        Err(_) => return -1,
    };

    let client = unsafe { &mut *handle };
    let room = client.room.clone();
    let r_id = RoomId::new(room_id_str);
    let p_name = peer_name_str.to_string();

    let res = client
        .runtime
        .block_on(async move { room.join(r_id, p_name, None).await });

    if res.is_ok() {
        0
    } else {
        -1
    }
}

/// Publish a track. Kind: 0 = Audio, 1 = Video.
#[no_mangle]
pub extern "C" fn alvio_client_publish_track(
    handle: *mut AlvioClientHandle,
    kind: i32,
    source: *const c_char,
) -> i32 {
    if handle.is_null() || source.is_null() {
        return -1;
    }

    let stream_kind = match kind {
        0 => StreamKind::Audio,
        1 => StreamKind::Video,
        _ => return -1,
    };

    let source_str = match unsafe { CStr::from_ptr(source) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -1,
    };

    let client = unsafe { &mut *handle };
    let room = client.room.clone();

    let layers = if stream_kind == StreamKind::Video {
        vec![StreamLayer::Low, StreamLayer::Medium, StreamLayer::High]
    } else {
        vec![StreamLayer::High]
    };

    let res = client
        .runtime
        .block_on(async move { room.publish_track(stream_kind, source_str, layers).await });

    if res.is_ok() {
        0
    } else {
        -1
    }
}

/// Send application data message to room. Returns 0 on success, -1 on failure.
#[no_mangle]
pub extern "C" fn alvio_client_send_data(
    handle: *mut AlvioClientHandle,
    payload: *const c_char,
    reliable: bool,
) -> i32 {
    if handle.is_null() || payload.is_null() {
        return -1;
    }

    let payload_str = match unsafe { CStr::from_ptr(payload) }.to_str() {
        Ok(s) => s.to_string(),
        Err(_) => return -1,
    };

    let client = unsafe { &mut *handle };
    let room = client.room.clone();

    let res = client
        .runtime
        .block_on(async move { room.send_data_message(vec![], payload_str, reliable).await });

    if res.is_ok() {
        0
    } else {
        -1
    }
}

/// Voluntarily leave the active room. Returns 0 on success, -1 on failure.
#[no_mangle]
pub extern "C" fn alvio_client_leave(handle: *mut AlvioClientHandle) -> i32 {
    if handle.is_null() {
        return -1;
    }

    let client = unsafe { &mut *handle };
    let room = client.room.clone();

    let res = client.runtime.block_on(async move { room.leave().await });

    if res.is_ok() {
        0
    } else {
        -1
    }
}

/// Poll the next queued event as a JSON C-string. Returns NULL if queue is empty.
/// The caller MUST free non-null return values with `alvio_client_free_string`.
#[no_mangle]
pub extern "C" fn alvio_client_poll_event(handle: *mut AlvioClientHandle) -> *mut c_char {
    if handle.is_null() {
        return std::ptr::null_mut();
    }

    let client = unsafe { &mut *handle };
    match client.event_rx.try_recv() {
        Ok(event) => match serde_json::to_string(&event) {
            Ok(json) => match CString::new(json) {
                Ok(c_str) => c_str.into_raw(),
                Err(_) => std::ptr::null_mut(),
            },
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

/// Free a string previously allocated by `alvio_client_poll_event`.
#[no_mangle]
pub extern "C" fn alvio_client_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}
