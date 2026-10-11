pub mod browse;
pub mod reading;
pub mod xingce;

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::{App, Screen};

pub fn draw_menu(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = vec![
        ListItem::new(Line::from(format!(
            "行测刷题     {} 套 · 已作答 {} 题",
            app.papers.len(),
            app.records.answers.len()
        ))),
        ListItem::new(Line::from(format!(
            "时评素材     {} 篇 · 读过 {} 篇",
            app.articles.len(),
            app.records.read.len()
        ))),
        ListItem::new(Line::from(format!("常识         {} 条", app.sense.len()))),
        ListItem::new(Line::from(format!("时政         {} 条", app.news.len()))),
    ];
    let mut state = ListState::default();
    state.select(Some(app.menu_cursor));
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("读本 · 终端学习（↑↓ 选，回车进，q 退出）"),
        )
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(list, area, &mut state);
}

pub fn draw(f: &mut Frame, app: &App) {
    let area = f.area();
    match app.screen {
        Screen::Menu => draw_menu(f, app, area),
        Screen::XingcePapers | Screen::XingceScope | Screen::XingceQuestion => {
            xingce::draw(f, app, area)
        }
        Screen::Articles => reading::draw_list(f, app, area),
        Screen::ArticleBody => reading::draw_body(f, app, area),
        Screen::Sense | Screen::News => {
            let (label, entries) = if app.screen == Screen::Sense {
                ("常识", &app.sense)
            } else {
                ("时政", &app.news)
            };
            match app.entry_detail.and_then(|i| entries.get(i)) {
                Some(e) => browse::draw_entry_detail(f, e, area),
                None => browse::draw_entries(f, app, label, entries, area),
            }
        }
    }
}
