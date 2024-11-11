/// Lines are separated by newline characters or by line wrap.
/// * Documents have at least one line.
/// * Lines can be empty.
/// * Lines can touch.
pub fn calc_lines(
    galleys: &Galleys, ast: &AstTextRanges, text: &Text,
) -> Vec<(DocCharOffset, DocCharOffset)> {
    let mut result = vec![];
    let mut text_range_iter = ast.iter();
    for (galley_idx, galley) in galleys.galleys.iter().enumerate() {
        for (row_idx, _) in galley.galley.rows.iter().enumerate() {
            let start_cursor = galley
                .galley
                .from_rcursor(RCursor { row: row_idx, column: 0 });
            let row_start =
                galleys.char_offset_by_galley_and_cursor(galley_idx, &start_cursor, text);
            let end_cursor = galley.galley.cursor_end_of_row(&start_cursor);
            let row_end = galleys.char_offset_by_galley_and_cursor(galley_idx, &end_cursor, text);

            let mut range = (row_start, row_end);

            // rows in galley head/tail are excluded
            if row_end < galley.text_range().start() {
                continue;
            }
            if row_start > galley.text_range().end() {
                break;
            }

            // if the range bounds are in the middle of a syntax sequence, expand the range to include the whole sequence
            // this supports selecting a line that starts or ends with a syntax sequence that's captured until the selection happens
            for text_range in text_range_iter.by_ref() {
                if text_range.range.start() > range.end() {
                    break;
                }
                if text_range.range_type == AstTextRangeType::Text {
                    continue;
                }
                if text_range.range.contains_inclusive(range.0) {
                    range.0 = text_range.range.0;
                }
                if text_range.range.contains_inclusive(range.1) {
                    range.1 = text_range.range.1;
                    break;
                }
            }

            // bound row start and row end by the galley bounds
            let (min, max) = galley.text_range();
            range.0 = range.0.max(min).min(max);
            range.1 = range.1.max(min).min(max);

            result.push(range)
        }
    }

    result
}
