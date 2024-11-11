/// Text consists of all rendered text separated by captured syntax ranges. Every valid cursor position is in some text
/// range.
/// * Documents have at least one text range.
/// * Text ranges can be empty.
/// * Text ranges can touch.
pub fn calc_text(
    ast: &Ast, ast_ranges: &AstTextRanges, appearance: &Appearance, segs: &UnicodeSegs,
    selection: (DocCharOffset, DocCharOffset), selecting: bool, capture: &CaptureState,
) -> Vec<(DocCharOffset, DocCharOffset)> {
    let mut result = vec![];
    let mut last_range_pushed = false;
    for (i, text_range) in ast_ranges.iter().enumerate() {
        let captured = capture.captured(selection, ast, ast_ranges, i, selecting, appearance);

        let this_range_pushed = if !captured {
            // text range or uncaptured syntax range
            result.push(text_range.range);
            true
        } else {
            false
        };

        if !this_range_pushed && !last_range_pushed {
            // empty range between captured ranges
            result.push((text_range.range.0, text_range.range.0));
        }
        last_range_pushed = this_range_pushed;
    }

    if !last_range_pushed {
        // empty range at end of doc
        result.push((segs.last_cursor_position(), segs.last_cursor_position()));
    }
    if result.is_empty() {
        result = vec![(0.into(), 0.into())];
    }

    result
}
