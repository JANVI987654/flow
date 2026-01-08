use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

#[derive(Clone)]
struct Card {
    key: String,
    title: String,
    description: String,
}

struct Column {
    name: &'static str,
    cards: Vec<Card>,
}

struct App {
    cols: Vec<Column>,
    col: usize,
    row: usize,
    detail: bool,
}

impl App {
    fn new() -> Self {
        Self {
            cols: vec![
                Column {
                    name: "TO DO",
                    cards: vec![
                        Card {
                            key: "FLOW-1".into(),
                            title: "Add counts to column headers".into(),
                            description: "Show the number of issues in each column.".into(),
                        },
                        Card {
                            key: "FLOW-2".into(),
                            title: "Keyboard-first transitions".into(),
                            description: "Move selected issue left/right with one keystroke."
                                .into(),
                        },
                    ],
                },
                Column {
                    name: "IN PROGRESS",
                    cards: vec![
                        Card {
                            key: "FLOW-3".into(),
                            title: "Detail pane / modal".into(),
                            description: "Enter toggles detail; it follows selection.".into(),
                        },
                        Card {
                            key: "FLOW-4".into(),
                            title: "Polish focused column styling".into(),
                            description: "Subtle focus color; readable defaults.".into(),
                        },
                    ],
                },
                Column {
                    name: "IN REVIEW",
                    cards: vec![Card {
                        key: "FLOW-5".into(),
                        title: "Demo data realism pass".into(),
                        description: "Keep demo neutral and screenshot-friendly.".into(),
                    }],
                },
                Column {
                    name: "DONE",
                    cards: vec![Card {
                        key: "FLOW-6".into(),
                        title: "Initial ratatui scaffold".into(),
                        description: "Basic columns + selection + transitions.".into(),
                    }],
                },
            ],
            col: 0,
            row: 0,
            detail: false,
        }
    }

    fn clamp_row(&mut self) {
        let len = self.cols[self.col].cards.len();
        self.row = match len {
            0 => 0,
            _ => self.row.min(len - 1),
        };
    }

    fn focus(&mut self, delta: isize) {
        let max = self.cols.len() as isize - 1;
        self.col = (self.col as isize + delta).clamp(0, max) as usize;
        self.clamp_row();
    }

    fn select(&mut self, delta: isize) {
        let len = self.cols[self.col].cards.len();
        if len == 0 {
            self.row = 0;
            return;
        }
        self.row = (self.row as isize + delta).clamp(0, len as isize - 1) as usize;
    }

    fn move_card(&mut self, dir: isize) {
        let dst = self.col as isize + dir;
        if dst < 0 || dst >= self.cols.len() as isize {
            return;
        }
        if self.cols[self.col].cards.is_empty() {
            return;
        }

        let dst = dst as usize;
        let card = self.cols[self.col].cards.remove(self.row);
        self.cols[dst].cards.push(card);

        self.col = dst;
        self.clamp_row();
        if !self.cols[self.col].cards.is_empty() {
            self.row = self.cols[self.col].cards.len() - 1;
        }
    }

    fn focused_card(&self) -> Option<&Card> {
        self.cols.get(self.col).and_then(|c| c.cards.get(self.row))
    }
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run(&mut terminal);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    res
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|f| render(f, &app))?;
        if handle_events(&mut app)? {
            break Ok(());
        }
    }
}

fn handle_events(app: &mut App) -> io::Result<bool> {
    match event::read()? {
        Event::Key(k) if k.kind == KeyEventKind::Press => match k.code {
            KeyCode::Char('q') => return Ok(true),

            KeyCode::Esc => {
                if app.detail {
                    app.detail = false;
                } else {
                    return Ok(true);
                }
            }

            KeyCode::Char('h') | KeyCode::Left => app.focus(-1),
            KeyCode::Char('l') | KeyCode::Right => app.focus(1),

            KeyCode::Char('j') | KeyCode::Down => app.select(1),
            KeyCode::Char('k') | KeyCode::Up => app.select(-1),

            KeyCode::Char('H') => app.move_card(-1),
            KeyCode::Char('L') => app.move_card(1),

            KeyCode::Enter => app.detail = !app.detail,

            _ => {}
        },
        _ => {}
    }
    Ok(false)
}

fn render(f: &mut Frame, app: &App) {
    let [main, help] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(2)])
        .split(f.area())[..]
        .try_into()
        .unwrap();

    let rects = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
            Constraint::Ratio(1, app.cols.len() as u32);
            app.cols.len()
        ])
        .split(main);

    for (i, r) in rects.iter().enumerate() {
        draw_col(f, app, i, *r);
    }

    f.render_widget(
        Paragraph::new("h/l focus  j/k select  H/L move  Enter detail  Esc close/quit  q quit")
            .block(Block::default().borders(Borders::TOP)),
        help,
    );

    if app.detail {
        if let Some(card) = app.focused_card() {
            let area = centered(70, 45, f.area());
            f.render_widget(Clear, area);
            f.render_widget(
                Paragraph::new(vec![
                    Line::from(Span::styled(
                        &card.key,
                        Style::default().add_modifier(Modifier::BOLD),
                    )),
                    Line::from(""),
                    Line::from(card.title.clone()),
                    Line::from(""),
                    Line::from(card.description.clone()),
                ])
                .wrap(Wrap { trim: true })
                .block(
                    Block::default()
                        .title("Detail")
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::DarkGray)),
                ),
                area,
            );
        }
    }
}

fn draw_col(f: &mut Frame, app: &App, idx: usize, rect: Rect) {
    let col = &app.cols[idx];
    let focused = idx == app.col;

    let border = if focused {
        Color::Cyan
    } else if col.name == "Done" {
        Color::DarkGray
    } else {
        Color::Gray
    };

    let items: Vec<ListItem> = col
        .cards
        .iter()
        .map(|c| {
            ListItem::new(Line::from(vec![
                Span::styled(&c.key, Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::raw(c.title.clone()),
            ]))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(format!("{} ({})", col.name, col.cards.len()))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border)),
        )
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = ListState::default();
    if focused && !col.cards.is_empty() {
        state.select(Some(app.row.min(col.cards.len() - 1)));
    }

    f.render_stateful_widget(list, rect, &mut state);
}

fn centered(px: u16, py: u16, r: Rect) -> Rect {
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - py) / 2),
            Constraint::Percentage(py),
            Constraint::Percentage((100 - py) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - px) / 2),
            Constraint::Percentage(px),
            Constraint::Percentage((100 - px) / 2),
        ])
        .split(v[1])[1]
}
