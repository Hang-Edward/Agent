use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use tauri::Emitter;

pub struct TerminalSession {
    process: Mutex<Option<Child>>,
}

impl TerminalSession {
    pub fn new() -> Self {
        Self {
            process: Mutex::new(None),
        }
    }

    pub fn start(&self, app: &tauri::AppHandle) -> Result<(), String> {
        let mut proc = self.process.lock().map_err(|e| format!("锁定失败: {}", e))?;
        if proc.is_some() {
            return Ok(());
        }

        let mut child = Command::new("powershell")
            .args(["-NoLogo", "-NoProfile"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("启动终端失败: {}", e))?;

        // 在 child 移入 Mutex 之前，先取出管道
        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        *proc = Some(child);

        // 读取 stdout
        if let Some(stdout) = stdout {
            let handle = app.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let _ = handle.emit("terminal:output", format!("{}\n", line));
                    }
                }
            });
        }

        // 读取 stderr
        if let Some(stderr) = stderr {
            let handle = app.clone();
            std::thread::spawn(move || {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        let _ = handle.emit("terminal:output", format!("{}\n", line));
                    }
                }
            });
        }

        Ok(())
    }

    pub fn write(&self, input: &str) -> Result<(), String> {
        let mut proc = self.process.lock().map_err(|e| format!("锁定失败: {}", e))?;
        if let Some(ref mut child) = *proc {
            if let Some(stdin) = child.stdin.as_mut() {
                writeln!(stdin, "{}", input).map_err(|e| format!("写入失败: {}", e))?;
                return Ok(());
            }
        }
        Err("终端未启动".to_string())
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut proc = self.process.lock().map_err(|e| format!("锁定失败: {}", e))?;
        if let Some(mut child) = proc.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        Ok(())
    }
}
