//! 常识 / 时政词条浏览视图。

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::App;
use crate::data::entries::group_by;
use crate::data::types::Entry;
use crate::views::xingce::wrap;

pub fn draw_entries(f: &mut Frame, app: &App, label: &str, entries: &[Entry], area: Rect) {
    if entries.is_empty() {
        let p = Paragraph::new(format!(
            "{label}词条库尚无数据。\n\n数据规格见主仓《行测终端刷题方案.md》§3.3：\n  data/{}.json —— term / group / meaning / distinction / example\n\n（esc 返回）",
            if label == "常识" { "common-sense" } else { "politics-hot" }
        ))
        .block(Block::default().borders(Borders::ALL).title(format!("{label}（待造数据）")));
        f.render_widget(p, area);
        return;
    }

    let groups = group_by(entries);
    let mut items: Vec<ListItem> = Vec::new();
    for (g, list) in &groups {
        items.push(ListItem::new(Line::from(Span::styled(
            format!("【{g}】"),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ))));
        for e in list {
            items.push(ListItem::new(Line::from(format!("  {}", e.term))));
        }
    }
    let mut state = ListState::default();
    state.select(Some(app.entry_cursor.min(items.len().saturating_sub(1))));
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("{label}（↑↓ 选，回车看，esc 返回）")),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_stateful_widget(list, area, &mut state);
}

pub fn draw_entry_detail(f: &mut Frame, e: &Entry, area: Rect) {
    let width = area.width.saturating_sub(4) as usize;
    let mut lines = vec![
        Line::from(Span::styled(
            &e.term,
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("【{}】#{:03}", e.group, e.id),
            Style::default().fg(Color::DarkGray),
        )),
        Line::default(),
    ];
    for l in wrap(&e.meaning, width) {
        lines.push(Line::from(l));
    }
    if let Some(d) = &e.distinction {
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            "辨析",
            Style::default().fg(Color::Yellow),
        )));
        for l in wrap(d, width) {
            lines.push(Line::from(l));
        }
    }
    if let Some(x) = &e.example {
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            "例",
            Style::default().fg(Color::Yellow),
        )));
        for l in wrap(x, width) {
            lines.push(Line::from(l));
        }
    }
    let p = Paragraph::new(Text::from(lines)).block(
        Block::default()
            .borders(Borders::ALL)
            .title("回车 / esc 返回列表"),
    );
    f.render_widget(p, area);
}
