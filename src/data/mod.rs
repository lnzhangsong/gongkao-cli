//! 数据加载入口：路径解析 + 三条加载线（行测 / 时评 / 词条）。
//!
//! 所有源都只读主仓 `data/`，不复制、不改动。

pub mod articles;
pub mod entries;
pub mod types;
pub mod xingce;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// 主仓 `data/` 目录的定位结果。
///
/// `root` 指向主仓的 `data/`（行测题库、时评库）；`entries` 指向词条库所在目录——
/// 常识/时政是本仓自己的数据（`gongkao-cli/data/`），不复用主仓。
#[derive(Debug, Clone)]
pub struct DataDir {
    pub root: PathBuf,
    pub entries: PathBuf,
}

impl DataDir {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        // 词条库优先找本仓 `data/`（即 root 的兄弟目录中，本仓自己的 data）；
        // 找不到就退回主仓 data/，保持「--data 只指一处」的简单心智。
        let entries = locate_entries_dir(&root).unwrap_or_else(|| root.clone());
        Self { root, entries }
    }

    pub fn xingce_dir(&self) -> PathBuf {
        self.root.join("xingce")
    }

    /// 时评所在的库。它是**构建产物**（`.gitignore` 未入库），需先在主仓 `vp run db:rebuild`。
    pub fn articles_db(&self) -> PathBuf {
        self.root.join("articles.db")
    }

    pub fn entries_file(&self, name: &str) -> PathBuf {
        self.entries.join(name)
    }

    /// 校验三条线的必要文件，缺失时给出可操作的报错。
    pub fn validate(&self) -> Result<()> {
        if !self.xingce_dir().is_dir() {
            anyhow::bail!("缺少行测题库目录：{}", self.xingce_dir().display());
        }
        if !self.articles_db().is_file() {
            anyhow::bail!(
                "缺少时评库：{}\n它是构建产物，请先在主仓跑 `vp run db:rebuild`",
                self.articles_db().display()
            );
        }
        Ok(())
    }
}

/// 读一个 JSON 文件并反序列化，报错带上路径。
pub fn read_json<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = std::fs::read(path).with_context(|| format!("读不到 {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("解析失败 {}", path.display()))
}

/// 找词条库目录：先看主仓 data 的兄弟目录里有没有本仓的 `data/`（含 common-sense.json）。
/// 词条是本仓自带数据，不进主仓，故不能只认 `--data` 指的那个目录。
fn locate_entries_dir(root: &Path) -> Option<PathBuf> {
    // 1) 主仓 data 的同级 `gongkao-cli/data/`
    if let Some(parent) = root.parent() {
        let sibling = parent.join("gongkao-cli/data");
        if sibling.join("common-sense.json").is_file() {
            return Some(sibling);
        }
    }
    // 2) 可执行文件所在目录上溯找 `data/common-sense.json`（cargo run / release 安装）
    if let Ok(exe) = std::env::current_exe() {
        for up in exe.ancestors().skip(1).take(3) {
            let cand = up.join("data");
            if cand.join("common-sense.json").is_file() {
                return Some(cand);
            }
        }
    }
    // 3) 开发时从 CWD 找
    let cwd_data = Path::new("data");
    if cwd_data.join("common-sense.json").is_file() {
        return Some(cwd_data.to_path_buf());
    }
    None
}
