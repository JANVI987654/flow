#[derive(Clone, PartialEq, Eq)]
pub struct Card {
    pub id: String,
    pub title: String,
    pub description: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Column {
    pub id: String,
    pub title: String,
    pub cards: Vec<Card>,
}

#[derive(Clone, PartialEq, Eq)]
pub struct Board {
    pub columns: Vec<Column>,
}
