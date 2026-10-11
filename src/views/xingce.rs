//! 行测刷题视图：选卷 → 选题范围 → 逐题作答 → 判分 → 解析。

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::{App, Screen};

pub fn draw_papers(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .papers
        .iter()
        .map(|p| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:>4}  ", p.year), Style::default().fg(Color::Cyan)),
                Span::raw(format!("{}  ", p.level)),
                Span::raw(format!("{}  ", p.title)),
                Span::styled(
                    format!("可刷 {} / 跳过 {}", p.practicable, p.skipped),
                    Style::default().fg(Color::DarkGray),
                ),
            ]))
        })
        .collect();
    let mut state = ListState::default();
    state.select(Some(app.paper_cursor));
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("行测 · 选卷（↑↓ 选，回车进，esc 返回）"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_stateful_widget(list, area, &mut state);
}

pub fn draw_scope(f: &mut Frame, app: &App, area: Rect) {
    let scopes = ["全部可刷题", "未做过", "做错的"];
    let items: Vec<ListItem> = scopes.iter().map(|s| ListItem::new(*s)).collect();
    let mut state = ListState::default();
    state.select(Some(app.scope_cursor));
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("题范围（回车开始，esc 返回）"),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));
    f.render_stateful_widget(list, area, &mut state);
}

pub fn draw_question(f: &mut Frame, app: &App, area: Rect) {
    let Some(q) = app.current_question() else {
        let p = Paragraph::new("本次已结束").block(Block::default().borders(Borders::ALL));
        f.render_widget(p, area);
        return;
    };
    let paper = app.loaded_paper.as_ref().unwrap();

    let chunks = Layout::vertical([
        Constraint::Min(6),
        Constraint::Length(if app.show_explanation { 12 } else { 0 }),
    ])
    .split(area);

    let answered = app.picked.is_some();
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        format!(
            "{} · 第 {} 题（{}/{}）",
            paper.title,
            q.idx,
            app.queue_pos + 1,
            app.queue.len()
        ),
        Style::default().fg(Color::Cyan),
    )));
    if let Some(section) = &q.section {
        lines.push(Line::from(Span::styled(
            format!("[{section}]"),
            Style::default().fg(Color::DarkGray),
        )));
    }
    lines.push(Line::default());
    if let Some(stem) = &q.group_stem {
        for l in wrap(stem, area.width.saturating_sub(4) as usize) {
            lines.push(Line::from(Span::styled(
                l,
                Style::default().fg(Color::DarkGray),
            )));
        }
        lines.push(Line::default());
    }
    for l in wrap(&q.stem, area.width.saturating_sub(4) as usize) {
        lines.push(Line::from(l));
    }
    lines.push(Line::default());
    for o in &q.options {
        let correct = answered && q.answer.as_deref() == Some(o.key.as_str());
        let wrong_pick = answered && app.picked.as_deref() == Some(o.key.as_str()) && !correct;
        let style = if correct {
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD)
        } else if wrong_pick {
            Style::default().fg(Color::Red)
        } else {
            Style::default()
        };
        for (i, l) in wrap(
            &format!("{}. {}", o.key, o.text),
            area.width.saturating_sub(4) as usize,
        )
        .into_iter()
        .enumerate()
        {
            let prefix = if i == 0 { "" } else { "   " };
            lines.push(Line::from(Span::styled(format!("{prefix}{l}"), style)));
        }
    }
    lines.push(Line::default());
    if answered {
        if q.answer.as_deref() == app.picked.as_deref() {
            lines.push(Line::from(Span::styled(
                "✅ 正确",
                Style::default().fg(Color::Green),
            )));
        } else {
            lines.push(Line::from(Span::styled(
                format!("❌ 错误，正确答案 {}", q.answer.as_deref().unwrap_or("?")),
                Style::default().fg(Color::Red),
            )));
        }
    }

    let hint = if answered {
        "回车看解析 / 下一条 · q 退出"
    } else {
        "1-4 作答 · q 退出"
    };
    let p =
        Paragraph::new(Text::from(lines)).block(Block::default().borders(Borders::ALL).title(hint));
    f.render_widget(p, chunks[0]);

    if app.show_explanation {
        let expl = q.explanation.as_deref().unwrap_or("（无解析）");
        let mut elines = Vec::new();
        for l in wrap(expl, area.width.saturating_sub(4) as usize) {
            elines.push(Line::from(l));
        }
        let p = Paragraph::new(Text::from(elines))
            .block(Block::default().borders(Borders::ALL).title("解析"));
        f.render_widget(p, chunks[1]);
    }
}

/// 按东亚字符宽度折行——中文题干若按字符数折会飘（方案 §五）。
pub fn wrap(s: &str, width: usize) -> Vec<String> {
    use unicode_width::UnicodeWidthStr;
    if width == 0 {
        return vec![s.to_string()];
    }
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut w = 0usize;
    for ch in s.chars() {
        if ch == '\n' {
            out.push(std::mem::take(&mut cur));
            w = 0;
            continue;
        }
        let cw = ch.to_string().as_str().width();
        if w + cw > width {
            out.push(std::mem::take(&mut cur));
            w = 0;
        }
        cur.push(ch);
        w += cw;
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    if out.is_empty() {
        out.push(String::new());
    }
    out
}

/// 由 main 的分发入口调用的绘制总入口，避免 Screen 逻辑散在各处。
pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    match app.screen {
        Screen::XingcePapers => draw_papers(f, app, area),
        Screen::XingceScope => draw_scope(f, app, area),
        Screen::XingceQuestion => draw_question(f, app, area),
        _ => {}
    }
}
