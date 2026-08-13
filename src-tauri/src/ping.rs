use futures::FutureExt;
use serde::{Deserialize, Serialize};
use std::process::{Command as StdCommand, Stdio};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Notify;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// Ping配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingConfig {
    pub targets: Vec<String>,
    pub packet_size: u32,
    pub interval: u32,
    pub timeout: u32,
    pub mode: String,
    pub count: u32,
}

// Ping结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingResult {
    pub target: String,
    pub ip: String,
    pub status: String,
    pub latency: f64,
    pub avg_latency: f64,
    pub max_latency: f64,
    pub min_latency: f64,
    pub loss_rate: f64,
    pub sent: u32,
    pub received: u32,
    pub timestamp: u64,
}

// 统计信息
#[derive(Debug, Clone, Default)]
struct PingStats {
    sent: u32,
    received: u32,
    latencies: Vec<f64>,
    current_latency: f64,
}

impl PingStats {
    fn add_result(&mut self, latency: f64, success: bool) {
        self.sent += 1;
        if success {
            self.received += 1;
            self.latencies.push(latency);
            self.current_latency = latency;
        } else {
            self.current_latency = 0.0;
        }
    }

    fn avg_latency(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.latencies.iter().sum();
        sum / self.latencies.len() as f64
    }

    fn max_latency(&self) -> f64 {
        self.latencies.iter().cloned().fold(0.0, f64::max)
    }

    fn min_latency(&self) -> f64 {
        if self.latencies.is_empty() {
            return 0.0;
        }
        self.latencies.iter().cloned().fold(f64::MAX, f64::min)
    }

    fn loss_rate(&self) -> f64 {
        if self.sent == 0 {
            return 0.0;
        }
        ((self.sent - self.received) as f64 / self.sent as f64) * 100.0
    }
}

// 解码命令输出（Windows用GBK，其他用UTF-8）
fn decode_output(bytes: &[u8]) -> String {
    #[cfg(target_os = "windows")]
    {
        let (decoded, _, _) = encoding_rs::GBK.decode(bytes);
        decoded.into_owned()
    }
    #[cfg(not(target_os = "windows"))]
    {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

// 同步执行命令（Windows下隐藏控制台窗口）
fn run_command(program: &str, args: &[String]) -> std::io::Result<std::process::Output> {
    let mut cmd = StdCommand::new(program);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    cmd.output()
}

// 解析ping输出中的延迟
fn parse_latency(output: &str) -> Option<f64> {
    // Windows中文: "时间=3ms" 或 "时间<1ms"
    // Windows英文: "time=3ms" 或 "time<1ms"
    // Linux/macOS: "time=3.5 ms"
    let re = regex::Regex::new(r"(?:时间|time)[=<]\s*(\d+(?:\.\d+)?)\s*ms").ok()?;
    re.captures(output)
        .and_then(|cap| cap[1].parse::<f64>().ok())
}

// 获取当前时间戳（毫秒）
fn timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// 执行单个目标的ping
pub async fn ping_target(
    app: AppHandle,
    target: String,
    config: PingConfig,
    cancel_notify: Arc<Notify>,
) -> Result<(), String> {
    let is_windows = cfg!(target_os = "windows");
    let mut stats = PingStats::default();

    let resolved_ip = resolve_target(&target).await.unwrap_or_else(|_| target.clone());
    let max_count = if config.mode == "count" { config.count } else { u32::MAX };
    let process_timeout = Duration::from_millis(config.timeout as u64 + 5000);

    for _seq in 0..max_count {
        if cancel_notify.notified().now_or_never().is_some() {
            break;
        }

        let (cmd, args) = build_ping_command(&target, config.packet_size, config.timeout, is_windows);
        let cmd_clone = cmd.clone();
        let args_clone = args.clone();

        let result = tokio::time::timeout(
            process_timeout,
            tokio::task::spawn_blocking(move || run_command(&cmd_clone, &args_clone)),
        )
        .await;

        match result {
            Ok(Ok(Ok(output))) => {
                let stdout = decode_output(&output.stdout);
                let success = output.status.success();
                let latency = if success {
                    parse_latency(&stdout).unwrap_or(0.0)
                } else {
                    0.0
                };

                let is_success = success && latency > 0.0;
                stats.add_result(latency, is_success);

                let ping_result = PingResult {
                    target: target.clone(),
                    ip: if resolved_ip.is_empty() { target.clone() } else { resolved_ip.clone() },
                    status: if is_success { "测试中".to_string() } else { "超时".to_string() },
                    latency,
                    avg_latency: (stats.avg_latency() * 100.0).round() / 100.0,
                    max_latency: (stats.max_latency() * 100.0).round() / 100.0,
                    min_latency: (stats.min_latency() * 100.0).round() / 100.0,
                    loss_rate: (stats.loss_rate() * 100.0).round() / 100.0,
                    sent: stats.sent,
                    received: stats.received,
                    timestamp: timestamp_ms(),
                };
                let _ = app.emit("ping-result", ping_result);
            }
            Ok(Ok(Err(e))) => {
                stats.add_result(0.0, false);
                let _ = app.emit("ping-result", make_result(&target, &resolved_ip, "失败", &stats));
                return Err(format!("执行ping失败: {}", e));
            }
            Ok(Err(e)) => {
                stats.add_result(0.0, false);
                let _ = app.emit("ping-result", make_result(&target, &resolved_ip, "失败", &stats));
                return Err(format!("执行ping失败: {}", e));
            }
            Err(_) => {
                stats.add_result(0.0, false);
                let _ = app.emit("ping-result", make_result(&target, &resolved_ip, "超时", &stats));
            }
        }

        tokio::time::sleep(Duration::from_millis(config.interval as u64)).await;
    }

    let final_result = PingResult {
        target: target.clone(),
        ip: resolved_ip,
        status: "完成".to_string(),
        latency: stats.current_latency,
        avg_latency: (stats.avg_latency() * 100.0).round() / 100.0,
        max_latency: (stats.max_latency() * 100.0).round() / 100.0,
        min_latency: (stats.min_latency() * 100.0).round() / 100.0,
        loss_rate: (stats.loss_rate() * 100.0).round() / 100.0,
        sent: stats.sent,
        received: stats.received,
        timestamp: timestamp_ms(),
    };
    let _ = app.emit("ping-result", final_result);

    Ok(())
}

fn make_result(target: &str, ip: &str, status: &str, stats: &PingStats) -> PingResult {
    PingResult {
        target: target.to_string(),
        ip: ip.to_string(),
        status: status.to_string(),
        latency: 0.0,
        avg_latency: (stats.avg_latency() * 100.0).round() / 100.0,
        max_latency: (stats.max_latency() * 100.0).round() / 100.0,
        min_latency: (stats.min_latency() * 100.0).round() / 100.0,
        loss_rate: (stats.loss_rate() * 100.0).round() / 100.0,
        sent: stats.sent,
        received: stats.received,
        timestamp: timestamp_ms(),
    }
}

fn build_ping_command(
    target: &str,
    packet_size: u32,
    timeout: u32,
    is_windows: bool,
) -> (String, Vec<String>) {
    if is_windows {
        (
            "ping".to_string(),
            vec![
                "-n".to_string(), "1".to_string(),
                "-w".to_string(), timeout.to_string(),
                "-l".to_string(), packet_size.to_string(),
                target.to_string(),
            ],
        )
    } else {
        let timeout_secs = (timeout / 1000).max(1).to_string();
        (
            "ping".to_string(),
            vec![
                "-c".to_string(), "1".to_string(),
                "-W".to_string(), timeout_secs,
                "-s".to_string(), packet_size.to_string(),
                target.to_string(),
            ],
        )
    }
}

pub async fn resolve_target(target: &str) -> Result<String, String> {
    if regex::Regex::new(r"^\d+\.\d+\.\d+\.\d+$")
        .unwrap()
        .is_match(target)
    {
        return Ok(target.to_string());
    }

    let target_clone = target.to_string();
    let output = tokio::task::spawn_blocking(move || {
        let cmd = if cfg!(target_os = "windows") { "nslookup" } else { "host" };
        run_command(cmd, &[target_clone])
    })
    .await
    .map_err(|e| format!("DNS解析失败: {}", e))?
    .map_err(|e| format!("DNS解析失败: {}", e))?;

    let stdout = decode_output(&output.stdout);
    let re = regex::Regex::new(r"(\d+\.\d+\.\d+\.\d+)").unwrap();
    for cap in re.captures_iter(&stdout) {
        let ip = &cap[1];
        if !ip.starts_with("127.") && !ip.starts_with("0.") {
            return Ok(ip.to_string());
        }
    }
    Err("无法解析域名".to_string())
}
