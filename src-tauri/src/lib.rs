mod ping;

use ping::PingConfig;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State};
use tokio::sync::Notify;
use tokio::task::JoinHandle;

// 应用状态
struct AppState {
    tasks: Mutex<Vec<JoinHandle<()>>>,
    cancel_notify: Arc<Notify>,
}

// 开始Ping
#[tauri::command]
async fn start_ping(
    app: AppHandle,
    state: State<'_, AppState>,
    config: PingConfig,
) -> Result<(), String> {
    // 先停止之前的任务
    stop_ping(state.clone()).await?;

    let targets = config.targets.clone();
    let cancel_notify = state.cancel_notify.clone();

    for target in targets {
        let app_clone = app.clone();
        let config_clone = config.clone();
        let cancel_clone = cancel_notify.clone();

        let handle = tokio::spawn(async move {
            let _ = ping::ping_target(app_clone, target, config_clone, cancel_clone).await;
        });

        state.tasks.lock().unwrap().push(handle);
    }

    Ok(())
}

// 停止Ping
#[tauri::command]
async fn stop_ping(state: State<'_, AppState>) -> Result<(), String> {
    // 通知所有任务取消
    state.cancel_notify.notify_waiters();

    // 等待所有任务完成
    let mut tasks = state.tasks.lock().unwrap();
    for task in tasks.drain(..) {
        task.abort();
    }

    Ok(())
}

// DNS解析
#[tauri::command]
async fn resolve_dns(target: String) -> Result<String, String> {
    ping::resolve_target(&target).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState {
        tasks: Mutex::new(Vec::new()),
        cancel_notify: Arc::new(Notify::new()),
    };

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![start_ping, stop_ping, resolve_dns])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
