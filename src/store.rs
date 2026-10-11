//! 作答与阅读记录的读写（JSON 落盘）。见主仓方案 §4.3。

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::data::types::Records;

/// 记录文件位置：`~/.local/share/gongkao-cli/records.json`。
/// `--data` 只管题库，不管记录——换题库不该丢记录。
pub fn records_path() -> Result<PathBuf> {
    let base = dirs::data_dir()
        .or_else(dirs::home_dir)
        .context("找不到用户数据目录")?;
    Ok(base.join("gongkao-cli/records.json"))
}

pub fn load(path: &Path) -> Result<Records> {
    if !path.is_file() {
        return Ok(Records {
            version: 1,
            ..Default::default()
        });
    }
    let bytes = std::fs::read(path).with_context(|| format!("读不到 {}", path.display()))?;
    let recs: Records = serde_json::from_slice(&bytes).unwrap_or(Records {
        version: 1,
        ..Default::default()
    });
    Ok(recs)
}

pub fn save(path: &Path, recs: &Records) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec_pretty(recs)?;
    std::fs::write(path, json).with_context(|| format!("写不了 {}", path.display()))?;
    Ok(())
}

pub fn reset(path: &Path) -> Result<()> {
    if path.is_file() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

/// 记录键形与 Web 端 `xg_answers` 一致：`paperId#qIdx`。
pub fn answer_key(paper_id: &str, q_idx: i64) -> String {
    format!("{paper_id}#{q_idx}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 键形一致() {
        assert_eq!(answer_key("p", 3), "p#3");
    }

    #[test]
    fn 读写往返() {
        let dir = std::env::temp_dir().join("gongkao-cli-test");
        let path = dir.join("records.json");
        let mut recs = Records {
            version: 1,
            ..Default::default()
        };
        recs.answers.insert(
            answer_key("p", 1),
            crate::data::types::AnswerRecord {
                paper_id: "p".into(),
                q_idx: 1,
                picked: "B".into(),
                correct: false,
                at: "2026-10-11T00:00:00".into(),
            },
        );
        save(&path, &recs).unwrap();
        let back = load(&path).unwrap();
        assert_eq!(back.answers.len(), 1);
        assert!(!back.answers["p#1"].correct);
        reset(&path).unwrap();
        assert!(load(&path).unwrap().answers.is_empty());
    }
}
