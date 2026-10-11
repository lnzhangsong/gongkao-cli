//! 应用状态机：主菜单 → 各板块 → 详情。

use std::path::PathBuf;

use anyhow::Result;

use crate::data::types::{Article, Entry, Paper, Records};
use crate::data::{articles, entries, xingce, DataDir};
use crate::store;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Menu,
    XingcePapers,
    XingceScope,
    XingceQuestion,
    Articles,
    ArticleBody,
    Sense,
    News,
}

pub struct App {
    pub data: DataDir,
    pub records: Records,
    pub records_path: PathBuf,
    pub screen: Screen,
    pub should_quit: bool,

    // 主菜单
    pub menu_cursor: usize,

    // 行测
    pub papers: Vec<xingce::PaperMeta>,
    pub paper_cursor: usize,
    pub loaded_paper: Option<Paper>,
    pub scope_cursor: usize,
    pub queue: Vec<usize>,
    pub queue_pos: usize,
    pub picked: Option<String>,
    pub show_explanation: bool,
    pub session_correct: usize,

    // 时评
    pub articles: Vec<Article>,
    pub article_cursor: usize,
    pub article_page: usize,
    pub topic_filter: Option<String>,

    // 常识 / 时政
    pub sense: Vec<Entry>,
    pub news: Vec<Entry>,
    pub entry_cursor: usize,
    pub entry_detail: Option<usize>,
}

impl App {
    pub fn new(root: PathBuf) -> Result<Self> {
        let data = DataDir::new(root);
        data.validate()?;
        let papers = xingce::list_papers(&data.xingce_dir())?;
        let arts = articles::list_articles(&data.articles_db())?;
        let sense = entries::load_entries(&data.entries_file("common-sense.json"))?;
        let news = entries::load_entries(&data.entries_file("politics-hot.json"))?;
        let rpath = store::records_path()?;
        let records = store::load(&rpath)?;

        Ok(Self {
            data,
            records,
            records_path: rpath,
            screen: Screen::Menu,
            should_quit: false,
            menu_cursor: 0,
            papers,
            paper_cursor: 0,
            loaded_paper: None,
            scope_cursor: 0,
            queue: Vec::new(),
            queue_pos: 0,
            picked: None,
            show_explanation: false,
            session_correct: 0,
            articles: arts,
            article_cursor: 0,
            article_page: 0,
            topic_filter: None,
            sense,
            news,
            entry_cursor: 0,
            entry_detail: None,
        })
    }

    pub fn enter(&mut self, screen: Screen) {
        self.screen = screen;
    }

    /// 加载当前卷，进入选题范围。
    pub fn open_paper(&mut self) -> Result<()> {
        if let Some(meta) = self.papers.get(self.paper_cursor) {
            let path = self.data.xingce_dir().join(format!("{}.json", meta.id));
            self.loaded_paper = Some(xingce::load_paper(&path)?);
            self.scope_cursor = 0;
            self.screen = Screen::XingceScope;
        }
        Ok(())
    }

    /// 按范围（全部可刷 / 未做过 / 做错的）构建队列。
    pub fn start_session(&mut self) {
        let Some(paper) = &self.loaded_paper else {
            return;
        };
        let scope = self.scope_cursor;
        let pid = &paper.id;
        self.queue = paper
            .questions
            .iter()
            .filter(|q| q.is_practicable())
            .filter(|q| match scope {
                1 => !self
                    .records
                    .answers
                    .contains_key(&store::answer_key(pid, q.idx)),
                2 => self
                    .records
                    .answers
                    .get(&store::answer_key(pid, q.idx))
                    .is_some_and(|r| !r.correct),
                _ => true,
            })
            .map(|q| q.idx as usize)
            .collect();
        self.queue_pos = 0;
        self.picked = None;
        self.show_explanation = false;
        self.session_correct = 0;
        self.screen = Screen::XingceQuestion;
    }

    pub fn current_question(&self) -> Option<&crate::data::types::Question> {
        let paper = self.loaded_paper.as_ref()?;
        let idx = *self.queue.get(self.queue_pos)? as i64;
        paper.questions.iter().find(|q| q.idx == idx)
    }

    /// 作答并落盘。
    pub fn answer(&mut self, key: &str) {
        let Some(paper) = &self.loaded_paper else {
            return;
        };
        let Some(q) = self.current_question() else {
            return;
        };
        let correct = q.answer.as_deref() == Some(key);
        let rec = crate::data::types::AnswerRecord {
            paper_id: paper.id.clone(),
            q_idx: q.idx,
            picked: key.to_string(),
            correct,
            at: now_iso(),
        };
        self.records
            .answers
            .insert(store::answer_key(&paper.id, q.idx), rec);
        if correct {
            self.session_correct += 1;
        }
        self.picked = Some(key.to_string());
        let _ = store::save(&self.records_path, &self.records);
    }

    pub fn next_question(&mut self) {
        if self.show_explanation {
            self.show_explanation = false;
        }
        self.picked = None;
        self.queue_pos += 1;
        if self.queue_pos >= self.queue.len() {
            self.screen = Screen::Menu;
        }
    }

    pub fn print_stats(&self) {
        println!(
            "行测：{} 套卷，已作答 {} 题",
            self.papers.len(),
            self.records.answers.len()
        );
        let correct = self.records.answers.values().filter(|r| r.correct).count();
        let total = self.records.answers.len();
        if total > 0 {
            println!(
                "累计正确率：{}/{} = {:.1}%",
                correct,
                total,
                correct as f64 / total as f64 * 100.0
            );
        }
        println!(
            "时评：{} 篇，读过 {} 篇",
            self.articles.len(),
            self.records.read.len()
        );
        println!(
            "常识：{} 条；时政：{} 条",
            self.sense.len(),
            self.news.len()
        );
    }
}

fn now_iso() -> String {
    // 记录里的时间戳只用于去重与展示，用 UNIX 秒即可，避免引入 chrono。
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_default()
}
