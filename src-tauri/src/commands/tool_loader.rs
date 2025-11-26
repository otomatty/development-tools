use std::path::PathBuf;
use std::fs;

use crate::types::{ToolConfig, ToolInfo};

/// toolsディレクトリのパスを取得
fn get_tools_dir() -> Result<PathBuf, String> {
    // 実行ファイルの場所から相対的にtoolsディレクトリを探す
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to get current executable path: {}", e))?;
    
    // 開発時は実行ファイルがtarget/debug等にあるので、プロジェクトルートを探す
    let mut search_path = current_exe.parent().map(|p| p.to_path_buf());
    
    while let Some(path) = search_path {
        let tools_dir = path.join("tools");
        if tools_dir.exists() && tools_dir.is_dir() {
            return Ok(tools_dir);
        }
        search_path = path.parent().map(|p| p.to_path_buf());
    }
    
    // フォールバック: カレントディレクトリからの相対パス
    let cwd = std::env::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;
    let tools_dir = cwd.join("tools");
    
    if tools_dir.exists() && tools_dir.is_dir() {
        Ok(tools_dir)
    } else {
        Err("Could not find tools directory".to_string())
    }
}

/// ツール一覧を取得するコマンド
#[tauri::command]
pub fn list_tools() -> Result<Vec<ToolInfo>, String> {
    let tools_dir = get_tools_dir()?;
    let mut tools = Vec::new();

    let entries = fs::read_dir(&tools_dir)
        .map_err(|e| format!("Failed to read tools directory: {}", e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let tool_json_path = path.join("tool.json");
        if !tool_json_path.exists() {
            continue;
        }

        match load_tool_config(&tool_json_path) {
            Ok(config) => {
                tools.push(ToolInfo {
                    name: config.name,
                    display_name: config.display_name,
                    description: config.description,
                    version: config.version,
                    icon: config.icon,
                    category: config.category,
                    tool_dir: path.to_string_lossy().to_string(),
                });
            }
            Err(e) => {
                eprintln!("Warning: Failed to load tool config at {:?}: {}", tool_json_path, e);
            }
        }
    }

    // 名前でソート
    tools.sort_by(|a, b| a.display_name.cmp(&b.display_name));

    Ok(tools)
}

/// 特定のツールの詳細設定を取得するコマンド
#[tauri::command]
pub fn get_tool_config(tool_name: String) -> Result<ToolConfig, String> {
    let tools_dir = get_tools_dir()?;
    let tool_json_path = tools_dir.join(&tool_name).join("tool.json");

    if !tool_json_path.exists() {
        return Err(format!("Tool '{}' not found", tool_name));
    }

    load_tool_config(&tool_json_path)
}

/// tool.jsonファイルを読み込んでパースする
fn load_tool_config(path: &PathBuf) -> Result<ToolConfig, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read tool.json: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse tool.json: {}", e))
}

/// ツールのバイナリパスを取得する
pub fn get_tool_binary_path(tool_name: &str) -> Result<PathBuf, String> {
    let tools_dir = get_tools_dir()?;
    let tool_dir = tools_dir.join(tool_name);
    let tool_json_path = tool_dir.join("tool.json");

    let config = load_tool_config(&tool_json_path)?;
    let binary_path = tool_dir.join(&config.binary);

    if !binary_path.exists() {
        return Err(format!(
            "Binary not found at {:?}. Please build the tool first.",
            binary_path
        ));
    }

    Ok(binary_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_tools_dir() {
        // This will work when run from the project root
        let result = get_tools_dir();
        println!("Tools dir: {:?}", result);
    }
}

