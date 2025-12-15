use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[allow(dead_code)]
pub fn cleanup_dir(package: &str) -> std::io::Result<()> {
    println!("[*] Cleaning up environment...");
    let mut cleanup_dir = vec![
        PathBuf::from("/storage/emulated/0/Android/data/").join(package),
        PathBuf::from("/storage/emulated/0/Android/media/").join(package),
    ];
    let datapath = Path::new("/data/data/").join(package);
    if !datapath.exists() {
        println!("[!] /data/data/{package} 不存在，跳过内部清理");
    } else {
        for entry in fs::read_dir(datapath)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();
            if name == "cache" || name == "code_cache" {
                let subdir = entry.path();
                for subentry in fs::read_dir(subdir)? {
                    let subentry = subentry?;
                    // println!("[+] add cleanup dir: {}", subentry.path().to_string_lossy());
                    cleanup_dir.push(subentry.path());
                }
            } else {
                // println!("[+] add cleanup dir: {}", entry.path().to_string_lossy());
                cleanup_dir.push(entry.path());
            }
        }
    }
    // Add cleanup code here
    cleanup_dir.iter().for_each(|p| {
        println!("[*] Removing directory {}", p.display());
        if p.is_dir() {
            // println!("清空目录 → {}", p.display());
            let _ = fs::remove_dir_all(p);
        } else {
            // println!("清空文件 → {}", p.display());
            let _ = fs::remove_file(p);
        }
    });

    Ok(())
}

#[allow(dead_code)]
pub fn reinstall() {
    match uninstall_apk() {
        Ok(msg) => {
            println!("[+] {msg}");
        }
        Err(e) => {
            println!("[-] Failed to uninstall: {e}");
        }
    }

    let apk = "./com.whatsapp.apk";
    let spoof_installer = "com.android.vending";
    match install_apk_with_spoof(apk, spoof_installer) {
        Ok(msg) => {
            println!("[+] msg: {msg}");
        }
        Err(e) => {
            println!("[-] Failed to reinstall: {e}");
        }
    }
}

/// 从 pm install-create 输出中提取 session id，
///
/// - "Success: created install session \[1294516556\]"
fn extract_session_id(output: &str) -> Option<String> {
    output
        .replace(['[', ']'], "")
        .split_whitespace()
        .find(|s| s.len() >= 5 && s.chars().all(char::is_numeric))
        .map(|s| s.to_string())
}

fn uninstall_apk() -> Result<String, String> {
    let uninstall_cmd = "pm uninstall com.whatsapp";
    let (success, output) = run_rook_command(&uninstall_cmd);
    if !success {
        Err(format!("{output}"))
    } else {
        Ok("Uninstalled successfully".to_string())
    }
}

/// 用 root + pm 静默安装（伪装 installer）
fn install_apk_with_spoof(apk_path: &str, installer_package: &str) -> Result<String, String> {
    let apk_path = Path::new(apk_path);

    if !apk_path.exists() {
        return Err("APK file does not exist".to_string());
    }

    // 1. 获取 APK 大小
    let total_size = apk_path
        .metadata()
        .map_err(|e| format!("Cannot get file size: {e}"))?
        .len();

    let created_cmd =
        format!("pm install-create -i {installer_package} --user 0 -r -S {total_size}");

    let (success, output) = run_rook_command(&created_cmd);
    if !success {
        return Err(format!("Failed to create session: {output}"));
    }

    let session_id = extract_session_id(&output)
        .ok_or_else(|| format!("Cannot extract session id from {output}"))?;

    println!("[+] Session created: {session_id}");

    let apk_abs = apk_path.canonicalize().map_err(|e| e.to_string())?;
    let apk_abs_str = apk_abs.to_string_lossy();

    let write_cmd = format!(
        "dd if=\"{apk_abs_str}\" | pm install-write -S {total_size} {session_id} \"{name}\"",
        name = apk_path.file_name().unwrap().to_string_lossy()
    );

    let (write_success, write_output) = run_rook_command(&write_cmd);
    if !write_success {
        let _ = run_rook_command(&format!("pm install-abandon {session_id}"));
        return Err(format!("Write failed: {write_output}"));
    }

    let commit_cmd = format!("pm install-commit {session_id}");
    let (commit_success, commit_output) = run_rook_command(&commit_cmd);
    if !commit_success {
        let _ = run_rook_command(&format!("pm install-abandon {session_id}"));
        return Err(format!("Commit failed: {commit_output}"));
    }

    return Ok(format!(
        "Installation succeeded (installer: {installer_package})"
    ));
}

#[allow(dead_code)]
fn run_rook_command(cmd: &str) -> (bool, String) {
    println!("[root] {cmd}");

    let output = Command::new("su")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout).to_string();
            let stderr = String::from_utf8_lossy(&out.stderr).to_string();
            let combined = format!("{stdout}\n{stderr}").to_string();

            let success = out.status.success();
            (success, combined)
        }
        Err(e) => (false, format!("Failed to start su: {e}")),
    }
}
