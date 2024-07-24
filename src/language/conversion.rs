//! Intermediate structures between tree-sitter and LSP types.

use tower_lsp::lsp_types as lsp;
use tree_sitter as ts;

/// Intermediate struct to convert between `ts::Point` and `lsp::Position`.
#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct Point(ts::Point);

impl Point {
    pub fn new(row: usize, column: usize) -> Self {
        Self(ts::Point { row, column })
    }

    pub fn zero() -> Self {
        Self(ts::Point { row: 0, column: 0 })
    }

    pub fn row(&self) -> usize {
        self.0.row
    }

    pub fn column(&self) -> usize {
        self.0.column
    }
}

impl From<Point> for ts::Point {
    fn from(value: Point) -> Self {
        value.0
    }
}

impl From<ts::Point> for Point {
    fn from(value: ts::Point) -> Self {
        Point(value)
    }
}

impl From<Point> for lsp::Position {
    fn from(value: Point) -> Self {
        lsp::Position::new(value.0.row as u32, value.0.column as u32)
    }
}

impl From<lsp::Position> for Point {
    fn from(value: lsp::Position) -> Self {
        Point(ts::Point {
            row: value.line as usize,
            column: value.character as usize,
        })
    }
}

/// Intermediate struct to convert between `ts::Range` and `lsp::Range`.
#[derive(Clone, Debug, PartialEq, PartialOrd, Ord, Eq)]
pub struct Range {
    start: Point,
    end: Point,
}

impl Range {
    pub fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }

    pub fn zero() -> Self {
        Self {
            start: Point::zero(),
            end: Point::zero(),
        }
    }
}

impl<'a> From<ts::Node<'a>> for Range {
    fn from(value: ts::Node<'a>) -> Self {
        Range {
            start: value.start_position().into(),
            end: value.end_position().into(),
        }
    }
}

impl From<ts::Range> for Range {
    fn from(value: ts::Range) -> Self {
        Range {
            start: value.start_point.into(),
            end: value.end_point.into(),
        }
    }
}

impl From<Range> for lsp::Range {
    fn from(value: Range) -> Self {
        lsp::Range::new(value.start.into(), value.end.into())
    }
}

impl From<lsp::Range> for Range {
    fn from(value: lsp::Range) -> Self {
        Range {
            start: value.start.into(),
            end: value.end.into(),
        }
    }
}
