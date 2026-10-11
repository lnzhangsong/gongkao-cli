//! 时评正文视图：列表 → 分段分页阅读。

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::views::xingce::wrap;

pub fn draw_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .articles
        .iter()
        .map(|a| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{}  ", a.date), Style::default().fg(Color::Cyan)),
                Span::raw(format!("{}  ", a.title)),
                Span::styled(&a.topic, Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.article_cursor));
    let title = match &app.topic_filter {
        Some(t) => format!("时评 · 主题「{t}」（↑↓ 选，回车读，esc 返回）"),
        None => format!(
            "时评 · {} 篇（↑↓ 选，回车读，esc 返回）",
            app.articles.len()
        ),
    };
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_stateful_widget(list, area, &mut state);
}

pub fn draw_body(f: &mut Frame, app: &App, area: Rect) {
    let Some(a) = app.articles.get(app.article_cursor) else {
        return;
    };
    let width = area.width.saturating_sub(4) as usize;
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            &a.title,
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("{} · {} · {}", a.source, a.column_name, a.date),
            Style::default().fg(Color::DarkGray),
        )),
        Line::default(),
    ];
    for p in a.paragraphs.iter().skip(app.article_page) {
        for (i, l) in wrap(p, width).into_iter().enumerate() {
            if i > 0 {
                lines.push(Line::from(format!("  {l}")));
            } else {
                lines.push(Line::from(format!("　{l}")));
            }
        }
        lines.push(Line::default());
    }
    let p = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title("j/k 逐段 · space/b 翻页 · esc 返回"),
    );
    f.render_widget(p, area);
}
