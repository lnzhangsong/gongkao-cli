use std::path::Path;

use anyhow::{Context, Result};
use rusqlite::Connection;

use super::types::Article;

/// 全部时评，按日期倒序。正文来自 `content_json`（段落字符串数组）二次解析。
pub fn list_articles(db: &Path) -> Result<Vec<Article>> {
    let conn = Connection::open_with_flags(db, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .with_context(|| format!("打不开时评库 {}", db.display()))?;
    let mut stmt = conn.prepare(
        "SELECT id, title, summary, source, topic, date, column_name, content_json
         FROM articles ORDER BY date DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        let content: String = row.get(7)?;
        Ok(Article {
            id: row.get(0)?,
            title: row.get(1)?,
            summary: row.get(2)?,
            source: row.get(3)?,
            topic: row.get(4)?,
            date: row.get(5)?,
            column_name: row.get(6)?,
            paragraphs: parse_content(&content),
        })
    })?;

    let mut out = Vec::new();
    for r in rows {
        let article = r?;
        // 空壳文章不入列表，见方案 §4.2。
        if article.is_readable() {
            out.push(article);
        }
    }
    Ok(out)
}

impl Article {
    pub fn is_readable(&self) -> bool {
        self.paragraphs.iter().any(|p| !p.trim().is_empty())
    }
}

/// `content_json` 是段落字符串的 JSON 数组；解析失败时退化为整段纯文本，不 panic。
fn parse_content(raw: &str) -> Vec<String> {
    match serde_json::from_str::<Vec<String>>(raw) {
        Ok(v) => v,
        Err(_) => vec![raw.to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 段落数组解析() {
        let v = parse_content("[\"第一段\",\"第二段\"]");
        assert_eq!(v, vec!["第一段", "第二段"]);
    }

    #[test]
    fn 坏_json_退化为纯文本() {
        let v = parse_content("不是 json");
        assert_eq!(v, vec!["不是 json"]);
    }

    #[test]
    fn 空段落文章不可读() {
        let a = Article {
            id: "x".into(),
            title: "t".into(),
            summary: String::new(),
            source: String::new(),
            topic: String::new(),
            date: String::new(),
            column_name: String::new(),
            paragraphs: vec!["  ".into()],
        };
        assert!(!a.is_readable());
    }
}
