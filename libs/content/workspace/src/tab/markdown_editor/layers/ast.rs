use comrak::{
    nodes::{LineColumn, NodeValue, Sourcepos},
    Arena, Options,
};
use lb_rs::text::offset_types::{DocCharOffset, RangeExt as _, RelCharOffset};

use crate::tab::markdown_editor::{self, utils};
use markdown_editor::Editor;

#[derive(Default)]
pub struct Ast {
    pub nodes: Vec<NodeValue>,
    pub ranges: Vec<(DocCharOffset, DocCharOffset)>,

    pub buffer_seq: usize,
    pub paragraphs_seq: usize,

    pub seq: usize,
}

impl Editor {
    pub fn ast(&mut self) {
        let Editor { buffer, paragraphs, ast, .. } = self;

        if utils::check_assign(&mut ast.buffer_seq, buffer.current.seq)
            || utils::check_assign(&mut ast.paragraphs_seq, paragraphs.seq)
        {
            ast.seq += 1;
        } else {
            return;
        }

        let arena = Arena::new();
        let root = comrak::parse_document(&arena, &buffer.current.text, &Options::default());

        ast.nodes.clear();
        for node in root.descendants() {
            let node = node.data.borrow();
            ast.nodes.push(node.value.clone());

            let Sourcepos {
                start: LineColumn { line: start_line, column: start_column },
                end: LineColumn { line: end_line, column: end_column },
            } = node.sourcepos;

            // lines and columns from comrak are 1-based
            let start_line = start_line - 1;
            let start_column = start_column - 1;
            let end_line = end_line - 1;
            let end_column = end_column - 1;

            // todo: are columns byte offsets or unicode char offsets?
            ast.ranges.push((
                paragraphs.ranges[start_line].start() + RelCharOffset(start_column),
                paragraphs.ranges[end_line].start() + RelCharOffset(end_column),
            ));
        }
    }
}
