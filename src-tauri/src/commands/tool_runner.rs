use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

use chrono::Utc;
use tauri::{AppHandle, Emitter};

use crate::commands::tool_loader::{get_tool_binary_path, get_tool_config};
use crate::types::{LogEvent, LogStream, OptionType, ToolResult, ToolStatus, ToolStatusEvent};

/// ツールを実行するコマンド
#[tauri::command]
pub async fn run_tool(
    app: AppHandle,
    tool_name: String,
    options: HashMap<String, serde_json::Value>,
) -> Result<(), String> {
    let binary_path = get_tool_binary_path(&tool_name)?;
    let config = get_tool_config(tool_name.clone())?;

    // コマンドライン引数を構築
    let mut args: Vec<String> = Vec::new();

    // JSON出力を強制（結果パースのため）
    if let Some(ref parser) = config.result_parser {
        if let Some(ref output_flag) = parser.output_flag {
            // "--output json" のような形式をパース
            for part in output_flag.split_whitespace() {
                args.push(part.to_string());
            }
        }
    }

    // ユーザーが指定したオプションを追加
    for opt_config in &config.options {
        if let Some(value) = options.get(&opt_config.name) {
            match opt_config.option_type {
                OptionType::Boolean => {
                    if let Some(b) = value.as_bool() {
                        if b {
                            args.push(opt_config.flag.clone());
                        }
                    }
                }
                OptionType::String | OptionType::Path => {
                    if let Some(s) = value.as_str() {
                        if !s.is_empty() {
                            args.push(opt_config.flag.clone());
                            // パスの場合、~をホームディレクトリに展開
                            let expanded = if opt_config.option_type == OptionType::Path {
                                expand_tilde(s)
                            } else {
                                s.to_string()
                            };
                            args.push(expanded);
                        }
                    }
                }
                OptionType::Select => {
                    if let Some(s) = value.as_str() {
                        // outputオプションはresult_parserで既に追加済みの場合スキップ
                        if opt_config.name != "output" || config.result_parser.is_none() {
                            args.push(opt_config.flag.clone());
                            args.push(s.to_string());
                        }
                    }
                }
                OptionType::Number => {
                    if let Some(n) = value.as_f64() {
                        args.push(opt_config.flag.clone());
                        args.push(n.to_string());
                    }
                }
            }
        }
    }

    // 実行開始を通知
    emit_status(&app, &tool_name, ToolStatus::Running, None);

    // 別スレッドでプロセスを実行
    let tool_name_clone = tool_name.clone();
    let app_clone = app.clone();
    let cancelled = Arc::new(AtomicBool::new(false));

    thread::spawn(move || {
        let result = execute_process(&app_clone, &tool_name_clone, &binary_path, &args, &cancelled);

        match result {
            Ok(tool_result) => {
                let status = if tool_result.success {
                    ToolStatus::Completed
                } else {
                    ToolStatus::Failed
                };
                emit_status(&app_clone, &tool_name_clone, status, Some(tool_result));
            }
            Err(e) => {
                let error_result = ToolResult {
                    success: false,
                    exit_code: None,
                    stdout: String::new(),
                    stderr: e,
                    parsed_result: None,
                };
                emit_status(&app_clone, &tool_name_clone, ToolStatus::Failed, Some(error_result));
            }
        }
    });

    Ok(())
}

/// プロセスを実行し、出力をリアルタイムで送信
fn execute_process(
    app: &AppHandle,
    tool_name: &str,
    binary_path: &std::path::Path,
    args: &[String],
    _cancelled: &Arc<AtomicBool>,
) -> Result<ToolResult, String> {
    let mut child = Command::new(binary_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start process: {}", e))?;

    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

    let mut stdout_content = String::new();
    let mut stderr_content = String::new();

    // stdoutを読み取るスレッド
    let app_stdout = app.clone();
    let tool_name_stdout = tool_name.to_string();
    let stdout_handle = thread::spawn(move || {
        let reader = BufReader::new(stdout);
        let mut content = String::new();
        for line in reader.lines().map_while(Result::ok) {
            emit_log(&app_stdout, &tool_name_stdout, &line, LogStream::Stdout);
            content.push_str(&line);
            content.push('\n');
        }
        content
    });

    // stderrを読み取るスレッド
    let app_stderr = app.clone();
    let tool_name_stderr = tool_name.to_string();
    let stderr_handle = thread::spawn(move || {
        let reader = BufReader::new(stderr);
        let mut content = String::new();
        for line in reader.lines().map_while(Result::ok) {
            emit_log(&app_stderr, &tool_name_stderr, &line, LogStream::Stderr);
            content.push_str(&line);
            content.push('\n');
        }
        content
    });

    // プロセスの終了を待つ
    let exit_status = child.wait().map_err(|e| format!("Failed to wait for process: {}", e))?;

    // スレッドの終了を待つ
    stdout_content = stdout_handle.join().unwrap_or_default();
    stderr_content = stderr_handle.join().unwrap_or_default();

    // JSON出力をパース
    let parsed_result = parse_json_output(&stdout_content);

    Ok(ToolResult {
        success: exit_status.success(),
        exit_code: exit_status.code(),
        stdout: stdout_content,
        stderr: stderr_content,
        parsed_result,
    })
}

/// JSON出力をパース
fn parse_json_output(output: &str) -> Option<serde_json::Value> {
    // 出力全体をJSONとしてパース
    if let Ok(value) = serde_json::from_str(output) {
        return Some(value);
    }

    // 行ごとに試す（NDJSON対応）
    let lines: Vec<&str> = output.lines().collect();
    if lines.len() == 1 {
        if let Ok(value) = serde_json::from_str(lines[0]) {
            return Some(value);
        }
    }

    // 最後の行を試す（プログレス出力後にJSONが来る場合）
    if let Some(last_line) = lines.last() {
        if let Ok(value) = serde_json::from_str(last_line) {
            return Some(value);
        }
    }

    None
}

/// ログイベントを送信
fn emit_log(app: &AppHandle, tool_name: &str, line: &str, stream: LogStream) {
    let event = LogEvent {
        tool_name: tool_name.to_string(),
        line: line.to_string(),
        stream,
        timestamp: Utc::now().to_rfc3339(),
    };
    let _ = app.emit("tool-log", event);
}

/// ステータスイベントを送信
fn emit_status(app: &AppHandle, tool_name: &str, status: ToolStatus, result: Option<ToolResult>) {
    let event = ToolStatusEvent {
        tool_name: tool_name.to_string(),
        status,
        result,
    };
    let _ = app.emit("tool-status", event);
}

/// ~をホームディレクトリに展開
fn expand_tilde(path: &str) -> String {
    if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            return path.replacen('~', &home.to_string_lossy(), 1);
        }
    }
    path.to_string()
}

