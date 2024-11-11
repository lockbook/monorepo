use comrak::nodes::NodeValue;
use lb_rs::text::{buffer::Buffer, offset_types::DocCharOffset};

use crate::tab::markdown_editor::{utils, Editor};

use super::ast::Ast;

/// Words are separated by UAX#29 (Unicode Standard Annex #29) word boundaries and do not contain whitespace. Some
/// punctuation marks count as words. Markdown syntax sequences count as single words.
/// * Documents may have no words.
/// * Words cannot be empty.
/// * Words can touch.
#[derive(Default)]
pub struct Words {
    pub ranges: Vec<(DocCharOffset, DocCharOffset)>,

    pub buffer_seq: usize,
    pub ast_seq: usize,

    pub seq: usize,
}

impl Editor {
    pub fn words(&mut self) {
        let Editor { buffer, ast, words, .. } = self;

        if utils::check_assign(&mut words.buffer_seq, buffer.current.seq)
            || utils::check_assign(&mut words.ast_seq, ast.seq)
        {
            words.seq += 1;
        } else {
            return;
        }

        // todo: handle captured syntax sequences
        words.ranges.clear();
        for node_idx in 0..ast.nodes.len() {
            let node = &ast.nodes[node_idx];
            let range = &ast.ranges[node_idx];

            match node {
                NodeValue::Document => todo!(),
                NodeValue::FrontMatter(front_matter) => todo!(),
                NodeValue::BlockQuote => todo!(),
                NodeValue::List(node_list) => todo!(),
                NodeValue::Item(node_list) => todo!(),
                NodeValue::DescriptionList => todo!(),
                NodeValue::DescriptionItem(node_description_item) => todo!(),
                NodeValue::DescriptionTerm => todo!(),
                NodeValue::DescriptionDetails => todo!(),
                NodeValue::CodeBlock(node_code_block) => todo!(),
                NodeValue::HtmlBlock(node_html_block) => todo!(),
                NodeValue::Paragraph => todo!(),
                NodeValue::Heading(node_heading) => todo!(),
                NodeValue::ThematicBreak => todo!(),
                NodeValue::FootnoteDefinition(node_footnote_definition) => todo!(),
                NodeValue::Table(node_table) => todo!(),
                NodeValue::TableRow(_) => todo!(),
                NodeValue::TableCell => todo!(),
                NodeValue::Text(_) => todo!(),
                NodeValue::TaskItem(_) => todo!(),
                NodeValue::SoftBreak => todo!(),
                NodeValue::LineBreak => todo!(),
                NodeValue::Code(node_code) => todo!(),
                NodeValue::HtmlInline(_) => todo!(),
                NodeValue::Emph => todo!(),
                NodeValue::Strong => todo!(),
                NodeValue::Strikethrough => todo!(),
                NodeValue::Superscript => todo!(),
                NodeValue::Link(node_link) => todo!(),
                NodeValue::Image(node_link) => todo!(),
                NodeValue::FootnoteReference(node_footnote_reference) => todo!(),
                NodeValue::Math(node_math) => todo!(),
                NodeValue::MultilineBlockQuote(node_multiline_block_quote) => todo!(),
                NodeValue::Escaped => todo!(),
                NodeValue::WikiLink(node_wiki_link) => todo!(),
                NodeValue::Underline => todo!(),
                NodeValue::SpoileredText => todo!(),
                NodeValue::EscapedTag(_) => todo!(),
            }
        }
    }
}

// todo: handle captured syntax sequences
pub fn calc_words(buffer: &Buffer, ast: &Ast) -> Vec<(DocCharOffset, DocCharOffset)> {
    for text_range in ast_ranges {
        if text_range.range_type != AstTextRangeType::Text
            && appearance.markdown_capture(text_range.node(ast).node_type())
                == CaptureCondition::Always
        {
            // skip always-captured syntax sequences
            continue;
        } else if text_range.range_type != AstTextRangeType::Text
            && !text_range.node(ast).node_type().syntax_includes_text()
        {
            // syntax sequences for node types without text count as single words
            result.push(text_range.range);
        } else {
            // remaining text and syntax sequences (including link URLs etc) are split into words
            let mut prev_char_offset = text_range.range.0;
            let mut prev_word = "";
            for (byte_offset, word) in
                (buffer[text_range.range].to_string() + " ").split_word_bound_indices()
            {
                let char_offset = buffer.current.segs.offset_to_char(
                    buffer.current.segs.offset_to_byte(text_range.range.0)
                        + RelByteOffset(byte_offset),
                );

                if !prev_word.trim().is_empty() {
                    // whitespace-only sequences don't count as words
                    result.push((prev_char_offset, char_offset));
                }

                prev_char_offset = char_offset;
                prev_word = word;
            }
        }
    }

    result
}
