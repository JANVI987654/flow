use crate::model::Board;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Quit,
    CloseOrQuit,
    FocusLeft,
    FocusRight,
    SelectUp,
    SelectDown,
    MoveLeft,
    MoveRight,
    ToggleDetail,
    Refresh,
}

pub struct App {
    pub board: Board,
    pub col: usize,
    pub row: usize,
    pub detail: bool,
    pub banner: Option<String>,
}

impl App {
    pub fn new(board: Board) -> Self {
        Self {
            board,
            col: 0,
            row: 0,
            detail: false,
            banner: None,
        }
    }

    fn clamp_row(&mut self) {
        let len = self
            .board
            .columns
            .get(self.col)
            .map(|c| c.cards.len())
            .unwrap_or(0);

        self.row = if len == 0 { 0 } else { self.row.min(len - 1) };
    }

    pub fn focus(&mut self, delta: isize) {
        if self.board.columns.is_empty() {
            self.col = 0;
            self.row = 0;
            return;
        }
        let max = self.board.columns.len() as isize - 1;
        self.col = (self.col as isize + delta).clamp(0, max) as usize;
        self.clamp_row();
    }

    pub fn select(&mut self, delta: isize) {
        if self.board.columns.is_empty() {
            self.row = 0;
            return;
        }
        let len = self.board.columns[self.col].cards.len();
        if len == 0 {
            self.row = 0;
            return;
        }
        self.row = (self.row as isize + delta).clamp(0, len as isize - 1) as usize;
    }

    pub fn apply(&mut self, a: Action) -> bool {
        match a {
            Action::Quit => return true,
            Action::CloseOrQuit => {
                if self.detail {
                    self.detail = false;
                } else {
                    return true;
                }
            }
            Action::FocusLeft => self.focus(-1),
            Action::FocusRight => self.focus(1),
            Action::SelectDown => self.select(1),
            Action::SelectUp => self.select(-1),
            Action::ToggleDetail => self.detail = !self.detail,
            Action::Refresh => {}
            Action::MoveLeft | Action::MoveRight => {}
        }
        false
    }

    pub fn optimistic_move(&mut self, dir: isize) -> Option<(String, String)> {
        if self.board.columns.is_empty() {
            return None;
        }
        let dst = self.col as isize + dir;
        if dst < 0 || dst >= self.board.columns.len() as isize {
            return None;
        }
        let dst = dst as usize;

        if self.board.columns[self.col].cards.is_empty() {
            return None;
        }

        let card = self.board.columns[self.col].cards.remove(self.row);
        let card_id = card.id.clone();
        let to_col_id = self.board.columns[dst].id.clone();

        self.board.columns[dst].cards.push(card);

        self.col = dst;
        self.clamp_row();
        if !self.board.columns[self.col].cards.is_empty() {
            self.row = self.board.columns[self.col].cards.len() - 1;
        }

        Some((card_id, to_col_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Board, Card, Column};

    #[test]
    fn move_right_moves_card() {
        let board = Board {
            columns: vec![
                Column {
                    id: "a".into(),
                    title: "A".into(),
                    cards: vec![Card {
                        id: "1".into(),
                        title: "t".into(),
                        description: "d".into(),
                    }],
                },
                Column {
                    id: "b".into(),
                    title: "B".into(),
                    cards: vec![],
                },
            ],
        };

        let mut app = App::new(board);
        let (id, dst) = app.optimistic_move(1).unwrap();

        assert_eq!(id, "1");
        assert_eq!(dst, "b");
        assert_eq!(app.board.columns[1].cards.len(), 1);
    }
}
