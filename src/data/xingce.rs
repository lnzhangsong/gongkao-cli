use std::path::Path;

use anyhow::Result;

use super::read_json;
use super::types::Paper;

/// 卷的元信息 + 可刷题统计，供列表页展示，避免把整卷题干都读进来。
#[derive(Debug, Clone)]
pub struct PaperMeta {
    pub id: String,
    pub year: i32,
    pub level: String,
    pub title: String,
    /// 可刷（文字）题数
    pub practicable: usize,
    /// 因含图或无答案被跳过的题数
    pub skipped: usize,
}

impl PaperMeta {
    pub fn from_paper(p: &Paper) -> Self {
        let practicable = p.questions.iter().filter(|q| q.is_practicable()).count();
        Self {
            id: p.id.clone(),
            year: p.year,
            level: p.level.clone(),
            title: p.title.clone(),
            practicable,
            skipped: p.questions.len() - practicable,
        }
    }
}

pub fn load_paper(path: &Path) -> Result<Paper> {
    read_json(path)
}

/// 目录下的全部卷，按年份倒序（2026 → 2000）。
pub fn list_papers(dir: &Path) -> Result<Vec<PaperMeta>> {
    let mut metas = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let paper: Paper = load_paper(&path)?;
        metas.push(PaperMeta::from_paper(&paper));
    }
    metas.sort_by(|a, b| b.year.cmp(&a.year).then(a.id.cmp(&b.id)));
    Ok(metas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::types::{ImageRef, Opt, Question};

    fn q(
        answer: Option<&str>,
        image: Option<Vec<ImageRef>>,
        group: Option<(&str, Option<&str>)>,
    ) -> Question {
        Question {
            idx: 1,
            section: None,
            subtype: None,
            group_id: group.map(|(id, _)| id.to_string()),
            group_stem: group.and_then(|(_, s)| s.map(str::to_string)),
            stem: "题干".into(),
            options: vec![Opt {
                key: "A".into(),
                text: "甲".into(),
            }],
            answer: answer.map(str::to_string),
            explanation: None,
            image,
            group_image: None,
        }
    }

    #[test]
    fn 有答案且无图可刷() {
        assert!(q(Some("B"), None, None).is_practicable());
    }

    #[test]
    fn 无答案跳过() {
        assert!(!q(None, None, None).is_practicable());
        assert!(!q(Some("  "), None, None).is_practicable());
    }

    #[test]
    fn 整题截图题跳过() {
        let img = vec![ImageRef {
            file: "q1_0.webp".into(),
        }];
        assert!(!q(Some("A"), Some(img), None).is_practicable());
    }

    #[test]
    fn 题组有文本材料可刷_无材料跳过() {
        assert!(q(Some("A"), None, Some(("g1", Some("材料正文")))).is_practicable());
        assert!(!q(Some("A"), None, Some(("g1", None))).is_practicable());
        assert!(!q(Some("A"), None, Some(("g1", Some("  ")))).is_practicable());
    }

    #[test]
    fn 跳过计数与可刷计数相加等于总题数() {
        let paper = Paper {
            id: "x".into(),
            year: 2025,
            level: "副省级".into(),
            title: "x".into(),
            duration_min: None,
            questions: vec![
                q(Some("A"), None, None),
                q(None, None, None),
                q(Some("B"), None, Some(("g1", Some("材料")))),
            ],
        };
        let meta = PaperMeta::from_paper(&paper);
        assert_eq!(meta.practicable, 2);
        assert_eq!(meta.skipped, 1);
    }

    /// 题库里 `groupId` 既有字符串也有整数（实测 999 题为整数）——不兼容会让整卷解析失败。
    #[test]
    fn groupId_整数与字符串都可解析() {
        let json = r#"[
          {"idx":1,"section":"资料分析","groupId":1,"groupStem":"材料","stem":"题干",
           "options":[{"key":"A","text":"甲"}],"answer":"A","explanation":""},
          {"idx":2,"section":"判断推理","groupId":"g9","groupStem":"材料","stem":"题干",
           "options":[{"key":"A","text":"甲"}],"answer":"B","explanation":""}
        ]"#;
        let qs: Vec<Question> = serde_json::from_str(json).unwrap();
        assert_eq!(qs[0].group_id.as_deref(), Some("1"));
        assert_eq!(qs[1].group_id.as_deref(), Some("g9"));
        assert!(qs.iter().all(|q| q.is_practicable()));
    }
}
