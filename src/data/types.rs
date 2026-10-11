use serde::{Deserialize, Serialize};

/// 行测一套卷。字段与主仓 `data/xingce/*.json` 一一对应。
/// 部分字段当前未用到，但保留完整结构便于后续功能（解析、字数、题组标号）。
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Paper {
    pub id: String,
    pub year: i32,
    pub level: String,
    pub title: String,
    #[serde(rename = "durationMin")]
    pub duration_min: Option<i64>,
    pub questions: Vec<Question>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Question {
    pub idx: i64,
    pub section: Option<String>,
    pub subtype: Option<String>,
    /// 题组编号。题库里既有字符串也有整数（实测 999 题为整数），故统一按值转文本。
    #[serde(rename = "groupId", deserialize_with = "de_opt_id")]
    pub group_id: Option<String>,
    #[serde(rename = "groupStem")]
    pub group_stem: Option<String>,
    pub stem: String,
    pub options: Vec<Opt>,
    pub answer: Option<String>,
    pub explanation: Option<String>,
    pub image: Option<Vec<ImageRef>>,
    #[serde(rename = "groupImage")]
    pub group_image: Option<Vec<ImageRef>>,
}

#[derive(Debug, Deserialize)]
pub struct Opt {
    pub key: String,
    pub text: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ImageRef {
    pub file: String,
}

/// `groupId` 在题库里是字符串或整数混用，统一转成文本。
fn de_opt_id<'de, D>(d: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = Option::<serde_json::Value>::deserialize(d)?;
    Ok(match v {
        None | Some(serde_json::Value::Null) => None,
        Some(serde_json::Value::String(s)) => Some(s),
        Some(serde_json::Value::Number(n)) => Some(n.to_string()),
        Some(other) => Some(other.to_string()),
    })
}

impl Question {
    /// 可刷题：有答案、非整题截图题、非材料截图题，题组题须有文本材料。
    /// 判定口径见主仓 `docs/行测终端刷题方案.md` §4.1。
    pub fn is_practicable(&self) -> bool {
        let has_answer = self.answer.as_deref().is_some_and(|a| !a.trim().is_empty());
        let no_image = self.image.as_ref().is_none_or(|v| v.is_empty());
        let no_group_image = self.group_image.as_ref().is_none_or(|v| v.is_empty());
        let group_ok = self.group_id.as_deref().is_none_or(|_| {
            self.group_stem
                .as_deref()
                .is_some_and(|s| !s.trim().is_empty())
        });
        has_answer && no_image && no_group_image && group_ok
    }
}

/// `articles` 表的一行（时评）。`content_json` 段落数组在加载时二次解析。
#[allow(dead_code)]
#[derive(Debug)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub source: String,
    pub topic: String,
    pub date: String,
    pub column_name: String,
    pub paragraphs: Vec<String>,
}

/// 常识 / 时政词条（照 `data/idioms.json` 形态造）。
#[derive(Debug, Deserialize)]
pub struct Entry {
    pub id: i64,
    /// 词条名
    pub term: String,
    /// 所属分组（常识主题 / 时政月份）
    pub group: String,
    /// 释义 / 背景
    pub meaning: String,
    /// 常考辨析 / 易错点，可缺
    #[serde(default)]
    pub distinction: Option<String>,
    /// 关联表述 / 例句，可缺
    #[serde(default)]
    pub example: Option<String>,
}

/// 落盘的作答与阅读记录，见方案 §4.3。
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Records {
    pub version: u32,
    #[serde(default)]
    pub answers: std::collections::BTreeMap<String, AnswerRecord>,
    #[serde(default)]
    pub read: std::collections::BTreeMap<String, ReadRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnswerRecord {
    #[serde(rename = "paperId")]
    pub paper_id: String,
    #[serde(rename = "qIdx")]
    pub q_idx: i64,
    pub picked: String,
    pub correct: bool,
    pub at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadRecord {
    pub at: String,
    pub progress: usize,
}
