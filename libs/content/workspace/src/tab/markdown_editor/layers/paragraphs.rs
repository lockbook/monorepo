use std::collections::HashSet;

use lb_rs::text::offset_types::{DocByteOffset, DocCharOffset};

use crate::tab::markdown_editor::{utils, Editor};

/// Paragraphs are separated by newline characters.
/// * Documents have at least one paragraph.
/// * Paragraphs can be empty.
/// * Paragraphs cannot touch.
#[derive(Default)]
pub struct Paragraphs {
    pub ranges: Vec<(DocCharOffset, DocCharOffset)>,

    pub buffer_seq: usize,

    pub seq: usize,
}

impl Editor {
    pub fn paragraphs(&mut self) {
        let Editor { buffer, paragraphs, .. } = self;

        if utils::check_assign(&mut paragraphs.buffer_seq, buffer.current.seq) {
            paragraphs.seq += 1;
        } else {
            return;
        }

        paragraphs.ranges.clear();
        let carriage_return_matches = buffer
            .current
            .text
            .match_indices('\r')
            .map(|(idx, _)| DocByteOffset(idx))
            .collect::<HashSet<_>>();
        let line_feed_matches = buffer
            .current
            .text
            .match_indices('\n')
            .map(|(idx, _)| DocByteOffset(idx))
            .filter(|&byte_offset| !carriage_return_matches.contains(&(byte_offset - 1)));

        let mut newline_matches = Vec::new();
        newline_matches.extend(line_feed_matches);
        newline_matches.extend(carriage_return_matches);
        newline_matches.sort();

        let mut prev_char_offset = DocCharOffset(0);
        for byte_offset in newline_matches {
            let char_offset = buffer.current.segs.offset_to_char(byte_offset);
            paragraphs.ranges.push((prev_char_offset, char_offset));
            prev_char_offset = char_offset + 1 // skip the matched newline;
        }
        paragraphs
            .ranges
            .push((prev_char_offset, buffer.current.segs.last_cursor_position()));
    }
}
