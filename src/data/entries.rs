use std::path::Path;

use anyhow::Result;

use super::types::Entry;

/// 常识 / 时政词条库。文件缺失时返回空表（数据待造，空态由 UI 提示，不阻塞其他线）。
pub fn load_entries(path: &Path) -> Result<Vec<Entry>> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    super::read_json(path)
}

/// 按分组归类，保持插入顺序。
pub fn group_by(entries: &[Entry]) -> Vec<(String, Vec<&Entry>)> {
    let mut groups: Vec<(String, Vec<&Entry>)> = Vec::new();
    for e in entries {
        match groups.iter_mut().find(|(g, _)| g == &e.group) {
            Some((_, v)) => v.push(e),
            None => groups.push((e.group.clone(), vec![e])),
        }
    }
    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(id: i64, group: &str) -> Entry {
        Entry {
            id,
            term: format!("t{id}"),
            group: group.into(),
            meaning: "释义".into(),
            distinction: None,
            example: None,
        }
    }

    #[test]
    fn 分组保持顺序() {
        let entries = vec![e(1, "法律"), e(2, "经济"), e(3, "法律")];
        let g = group_by(&entries);
        assert_eq!(g.len(), 2);
        assert_eq!(g[0].0, "法律");
        assert_eq!(g[0].1.len(), 2);
        assert_eq!(g[1].0, "经济");
    }

    #[test]
    fn 缺文件返回空表() {
        let v = load_entries(Path::new("/definitely/not/here.json")).unwrap();
        assert!(v.is_empty());
    }
}
