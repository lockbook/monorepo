use linkify::LinkFinder;
use tldextract::{TldExtractor, TldOption};

use crate::tab::svg_editor::Buffer;

/// Plain text links are styled and clickable but aren't markdown links.
/// * Documents may have no links.
/// * Links cannot be empty.
/// * Links cannot touch.
pub fn calc_links(buffer: &Buffer, text: &Text, ast: &Ast) -> Vec<(DocCharOffset, DocCharOffset)> {
    let finder = {
        let mut this = LinkFinder::new();
        this.kinds(&[linkify::LinkKind::Url])
            .url_must_have_scheme(false)
            .url_can_be_iri(false); // ignore links with international characters for phishing prevention
        this
    };

    let mut result = vec![];
    for &text_range in text {
        'spans: for span in finder.spans(&buffer[text_range]) {
            let link_range = (text_range.0 + span.start(), text_range.0 + span.end());

            if span.kind().is_none() {
                continue;
            }

            let link_text = if buffer[link_range].contains("://") {
                buffer[link_range].to_string()
            } else {
                format!("http://{}", &buffer[link_range])
            };

            match TldExtractor::new(TldOption::default()).extract(&link_text) {
                Ok(tld) => {
                    // the last one of these must be a top level domain
                    if let Some(ref d) = tld.suffix {
                        if !tld::exist(d) {
                            continue;
                        }
                    } else if let Some(ref d) = tld.domain {
                        if !tld::exist(d) {
                            continue;
                        }
                    } else if let Some(ref d) = tld.subdomain {
                        if !tld::exist(d) {
                            continue;
                        }
                    }
                }
                Err(_) => {
                    continue;
                }
            }

            // ignore links in code blocks because field references or method invocations can look like URLs
            for node in &ast.nodes {
                let node_type_ignores_links = node.node_type.node_type()
                    == MarkdownNodeType::Block(BlockNodeType::Code)
                    || node.node_type.node_type() == MarkdownNodeType::Inline(InlineNodeType::Code);
                if node_type_ignores_links && node.range.intersects(&link_range, false) {
                    continue 'spans;
                }
            }

            result.push(link_range);
        }
    }

    result
}
