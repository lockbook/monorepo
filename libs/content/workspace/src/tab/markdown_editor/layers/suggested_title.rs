fn get_suggested_title(&self) -> Option<String> {
    if !self.needs_name {
        return None;
    }

    let ast_ranges = self
        .bounds
        .ast
        .iter()
        .map(|range| range.range)
        .collect::<Vec<_>>();
    for ([ast_idx, paragraph_idx], text_range_portion) in
        bounds::join([&ast_ranges, &self.bounds.paragraphs])
    {
        if let Some(ast_idx) = ast_idx {
            let ast_text_range = &self.bounds.ast[ast_idx];
            if ast_text_range.range_type != AstTextRangeType::Text {
                continue; // no syntax characters in suggested title
            }
            if ast_text_range.is_empty() {
                continue; // no empty text in suggested title
            }
        }
        if paragraph_idx > Some(0) {
            break; // suggested title must be from first paragraph
        }

        return Some(String::from(&self.buffer[text_range_portion]) + ".md");
    }
    None
}
