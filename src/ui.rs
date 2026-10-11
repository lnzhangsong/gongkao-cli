//! ratatui 主循环与事件分发。

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::app::{App, Screen};
use crate::views;

pub fn run(mut app: App) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    result
}

fn event_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| views::draw(f, app))?;
        if app.should_quit {
            return Ok(());
        }
        if let Event::Key(k) = event::read()? {
            if k.kind != KeyEventKind::Press {
                continue;
            }
            handle_key(app, k.code);
        }
    }
}

fn handle_key(app: &mut App, code: KeyCode) {
    if code == KeyCode::Char('q') {
        app.should_quit = true;
        return;
    }
    match app.screen {
        Screen::Menu => menu_key(app, code),
        Screen::XingcePapers => list_key(app, code, Move::Paper),
        Screen::XingceScope => scope_key(app, code),
        Screen::XingceQuestion => question_key(app, code),
        Screen::Articles => article_key(app, code),
        Screen::ArticleBody => body_key(app, code),
        Screen::Sense | Screen::News => entry_key(app, code),
    }
}

fn menu_key(app: &mut App, code: KeyCode) {
    let n = 4;
    match code {
        KeyCode::Up => app.menu_cursor = app.menu_cursor.saturating_sub(1),
        KeyCode::Down => app.menu_cursor = (app.menu_cursor + 1).min(n - 1),
        KeyCode::Enter => {
            app.screen = match app.menu_cursor {
                0 => Screen::XingcePapers,
                1 => Screen::Articles,
                2 => Screen::Sense,
                _ => Screen::News,
            }
        }
        _ => {}
    }
}

enum Move {
    Paper,
}

fn list_key(app: &mut App, code: KeyCode, _m: Move) {
    let n = app.papers.len();
    match code {
        KeyCode::Up => app.paper_cursor = app.paper_cursor.saturating_sub(1),
        KeyCode::Down => {
            if n > 0 {
                app.paper_cursor = (app.paper_cursor + 1).min(n - 1);
            }
        }
        KeyCode::Enter => {
            let _ = app.open_paper();
        }
        KeyCode::Esc => app.screen = Screen::Menu,
        _ => {}
    }
}

fn scope_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Up => app.scope_cursor = app.scope_cursor.saturating_sub(1),
        KeyCode::Down => app.scope_cursor = (app.scope_cursor + 1).min(2),
        KeyCode::Enter => app.start_session(),
        KeyCode::Esc => app.screen = Screen::XingcePapers,
        _ => {}
    }
}

fn question_key(app: &mut App, code: KeyCode) {
    if app.picked.is_none() {
        if let KeyCode::Char(c @ '1'..='4') = code {
            if let Some(q) = app.current_question() {
                let target = ((c as u8 - b'0') as usize - 1).min(q.options.len().saturating_sub(1));
                if let Some(opt) = q.options.get(target) {
                    let key = opt.key.clone();
                    app.answer(&key);
                }
            }
            return;
        }
    } else if code == KeyCode::Enter {
        if app.show_explanation {
            app.next_question();
        } else {
            app.show_explanation = true;
        }
        return;
    }
    if code == KeyCode::Esc {
        app.screen = Screen::Menu;
    }
}

fn article_key(app: &mut App, code: KeyCode) {
    let n = app.articles.len();
    match code {
        KeyCode::Up => app.article_cursor = app.article_cursor.saturating_sub(1),
        KeyCode::Down => {
            if n > 0 {
                app.article_cursor = (app.article_cursor + 1).min(n - 1);
            }
        }
        KeyCode::Enter => {
            app.article_page = 0;
            app.screen = Screen::ArticleBody;
        }
        KeyCode::Esc => app.screen = Screen::Menu,
        _ => {}
    }
}

fn body_key(app: &mut App, code: KeyCode) {
    let max = app
        .articles
        .get(app.article_cursor)
        .map(|a| a.paragraphs.len().saturating_sub(1))
        .unwrap_or(0);
    match code {
        KeyCode::Char('j') | KeyCode::Down => app.article_page = (app.article_page + 1).min(max),
        KeyCode::Char('k') | KeyCode::Up => app.article_page = app.article_page.saturating_sub(1),
        KeyCode::Char(' ') => app.article_page = (app.article_page + 5).min(max),
        KeyCode::Char('b') => app.article_page = app.article_page.saturating_sub(5),
        KeyCode::Esc => app.screen = Screen::Articles,
        _ => {}
    }
}

fn entry_key(app: &mut App, code: KeyCode) {
    let n = match app.screen {
        Screen::Sense => app.sense.len(),
        _ => app.news.len(),
    };
    // 详情态：只响应返回。
    if app.entry_detail.is_some() {
        if matches!(code, KeyCode::Esc | KeyCode::Enter) {
            app.entry_detail = None;
        }
        return;
    }
    match code {
        KeyCode::Up => app.entry_cursor = app.entry_cursor.saturating_sub(1),
        KeyCode::Down => {
            if n > 0 {
                app.entry_cursor = (app.entry_cursor + 1).min(n - 1);
            }
        }
        KeyCode::Enter => {
            if n > 0 {
                app.entry_detail = Some(app.entry_cursor.min(n - 1));
            }
        }
        KeyCode::Esc => app.screen = Screen::Menu,
        _ => {}
    }
}
