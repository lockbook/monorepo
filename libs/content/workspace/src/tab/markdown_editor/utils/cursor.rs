use crate::tab::markdown_editor;
use egui::{Pos2, Vec2};
use lb_rs::text::offset_types::DocCharOffset;
use lb_rs::text::offset_types::RangeExt as _;
use markdown_editor::appearance::Appearance;
use markdown_editor::bounds::Text;
use markdown_editor::galleys::{self, Galleys};

use super::advance::AdvanceExt as _;

pub fn line(
    offset: DocCharOffset, galleys: &Galleys, text: &Text, appearance: &Appearance,
) -> [Pos2; 2] {
    let (galley_idx, cursor) = galleys.galley_and_cursor_by_char_offset(offset, text);
    let galley = &galleys[galley_idx];

    let max = DocCharOffset::cursor_to_pos_abs(galley, cursor);
    let min = max - Vec2 { x: 0.0, y: galley.cursor_height() };

    if offset < galley.text_range().start() {
        // draw cursor before offset if that's where it is
        let annotation_offset = galleys::annotation_offset(&galley.annotation, appearance);
        let annotation_min_x = egui::vec2(annotation_offset.min.x, 0.);
        [min - annotation_min_x, max - annotation_min_x]
    } else {
        [min, max]
    }
}
