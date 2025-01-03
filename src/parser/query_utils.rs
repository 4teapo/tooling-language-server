use tower_lsp::lsp_types::{Position, Range};
use tree_sitter::Point;

pub fn point_to_position(point: Point) -> Position {
    Position {
        line: point.row as u32,
        character: point.column as u32,
    }
}

pub fn range_from_node(node: &tree_sitter::Node) -> Range {
    Range {
        start: point_to_position(node.start_position()),
        end: point_to_position(node.end_position()),
    }
}

pub fn range_contains(range: Range, pos: Position) -> bool {
    range.start <= pos && pos <= range.end
}

pub fn range_extend(range: Range, other: Range) -> Range {
    Range {
        start: std::cmp::min(range.start, other.start),
        end: std::cmp::max(range.end, other.end),
    }
}
