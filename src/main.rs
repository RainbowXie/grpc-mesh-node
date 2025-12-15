use clean::reinstall;

use frida::{Frida, ScriptHandler, ScriptOption, ScriptRuntime, SpawnOptions};
use serde_json;
use std::sync::LazyLock;

mod clean;
mod traffic;

use traffic::{FridaPayload, HttpMessage};

static FRIDA: LazyLock<Frida> = LazyLock::new(|| unsafe { Frida::obtain() });

struct Handler;

impl ScriptHandler for Handler {
    fn on_message(&mut self, message: frida::Message, data: Option<Vec<u8>>) {
        match message {
            frida::Message::Send(msg_send) => {
                println!("Received message: {:?}", msg_send);
            }
            frida::Message::Error(msg_error) => {
                println!("Received error: {:?}", msg_error);
            }
            frida::Message::Log(msg_log) => {
                println!("Received log: {:?}", msg_log);
            }
            frida::Message::Other(frida_value) => {
                process_frida_data(frida_value, data);
            }
        }
    }
}

fn process_frida_data(message: serde_json::Value, data: Option<Vec<u8>>) {
    // println!("[message]: {:#?}", message);
    let message = match message {
        serde_json::Value::Object(obj) => obj,
        _ => {
            eprintln!("Unexpected message type: {:?}", message);
            return;
        }
    };

    let message_data: serde_json::Value = match message.get("data") {
        Some(data) => data.clone(),
        None => {
            eprintln!("Missing 'data' field in message");
            return;
        }
    };

    let frida_payload_str = match message_data {
        serde_json::Value::String(obj) => {
            // println!("frida_playload_str: {}", obj);
            obj
        }
        _ => {
            eprintln!("Unexpected message type: {:?}", message);
            return;
        }
    };

    match serde_json::from_str::<FridaPayload>(&frida_payload_str) {
        Ok(frida_payload) => {
            if let Some(d) = &data {
                println!("[BODY BYTES]: {} bytes", d.len());
            }
            let http_message = HttpMessage {
                metadata: frida_payload.payload,
                body: data,
            };

            let body_str = match String::from_utf8(http_message.body.unwrap_or_default()) {
                Ok(s) => {
                    // 转换成功，s 是 String
                    Some(s)
                }
                Err(e) => {
                    // 转换失败 — bytes 不是有效 UTF‑8
                    eprintln!("Invalid UTF-8: {}", e);
                    None
                    // 你可以拿回原 Vec<u8>：e.into_bytes()
                }
            };
            println!(
                "[HTTP MESSAGE PARSED]: {:#?}\nbody: {}",
                http_message.metadata,
                body_str.unwrap_or_default()
            );
        }
        Err(e) => {
            eprintln!("Failed to deserialize metadata: {}", e);
            return;
        }
    };
}

fn main() {
    // let _ = cleanup_dir("com.whatsapp");
    // let _ = reinstall();
    frd()
}

#[allow(dead_code)]
fn frd() {
    println!("[*] Hello world! Frida version: {}", Frida::version());
    let device_manager = frida::DeviceManager::obtain(&FRIDA);
    // let devices = device_manager.enumerate_all_devices();

    // devices.iter().for_each(|device| {
    //     println!(
    //         "Device: {}, {}, {}",
    //         device.get_name(),
    //         device.get_type(),
    //         device.get_id(),
    //     );

    //     device.enumerate_processes().iter().for_each(|process| {
    //         println!("\tProcess: {}", process.get_name());
    //     });
    // });
    let mut device = device_manager.get_local_device().unwrap_or_else(|e| {
        eprintln!("[-] failed to get device: {}", e);
        std::process::exit(-1);
    });

    let pid = device
        .spawn("com.whatsapp", &SpawnOptions::new())
        .unwrap_or_else(|e| {
            eprintln!("[-] Failed to spawn process: {}", e);
            std::process::exit(-1);
        });

    println!("[*] attach");
    let session = device.attach(pid).unwrap_or_else(|e| {
        eprintln!("[-] Failed to attach to process: {}", e);
        std::process::exit(-1);
    });

    let mut script_option = ScriptOption::new()
        .set_name("wa")
        .set_runtime(ScriptRuntime::V8);

    println!("[*] create_script");
    let payload = include_str!("./assets/_tmp.js");
    let mut script = session
        .create_script(payload, &mut script_option)
        .unwrap_or_else(|e| {
            eprintln!("[-] Failed to create script: {}", e);
            std::process::exit(-1);
        });

    println!("[*] handle_message");
    script.handle_message(Handler).unwrap_or_else(|e| {
        eprintln!("[-] Failed to handle_message: {}", e);
        std::process::exit(-1);
    });

    println!("[*] load");
    script.load().unwrap_or_else(|e| {
        eprintln!("[-] Failed to load script: {}", e);
        std::process::exit(-1);
    });

    println!("[*] resume");
    device.resume(pid).unwrap_or_else(|e| {
        eprintln!("[-] Failed to resume process: {}", e);
        std::process::exit(-1)
    });

    println!("[*] Injected");
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .expect("Failed to read input");
}
