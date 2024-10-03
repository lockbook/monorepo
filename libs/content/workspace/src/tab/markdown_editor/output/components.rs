use egui::{text::LayoutJob, Color32, Pos2, Response, Sense, Stroke, Ui, Vec2};
use pulldown_cmark::HeadingLevel;

use crate::tab::markdown_editor::{
    style::{BlockNode, ListItem, MarkdownNode, RenderStyle},
    Editor,
};

pub struct Components {
    pub components: Vec<Component>,
}

pub struct Component {
    pub ast_node_idx: usize,
    pub response: ComponentResponse,
}

pub enum ComponentResponse {
    Paragraph { response: Response },
    Heading { text_response: Response, rule_response: Response },
    Quote { response: Response },
    CodeBlock { response: Response },
    Rule { response: Response },
    ListItem { marker_response: Response, text_response: Response },
    Image { response: Response },
}

impl Editor {
    // iterate ast nodes
    // block nodes create their own component; inline nodes dictate text styles within those
    pub fn calc_components(&self, ui: &mut Ui) -> Components {
        let mut components = Vec::new();

        self.rule(ui);

        for ast_node_idx in 0..self.ast.nodes.len() {
            let ast_node = &self.ast.nodes[ast_node_idx];

            match &ast_node.node_type {
                MarkdownNode::Document => {
                    // this just wraps everything else
                    continue;
                }
                MarkdownNode::Paragraph => {
                    let response = self.paragraph(ui);
                    components.push(Component {
                        ast_node_idx,
                        response: ComponentResponse::Paragraph { response },
                    });
                }
                MarkdownNode::Inline(_) => {
                    // todo: append to whatever we're in the middle of or something
                    continue;
                }
                MarkdownNode::Block(BlockNode::Heading(_level)) => {
                    let text_response = self.heading(ui);
                    let rule_response = self.rule(ui);
                    components.push(Component {
                        ast_node_idx,
                        response: ComponentResponse::Heading { text_response, rule_response },
                    });
                }
                MarkdownNode::Block(BlockNode::Quote) => {
                    let response = self.quote(ui);
                    components.push(Component {
                        ast_node_idx,
                        response: ComponentResponse::Quote { response },
                    });
                }
                MarkdownNode::Block(BlockNode::Code) => {
                    let response = self.code_block(ui);
                    components.push(Component {
                        ast_node_idx,
                        response: ComponentResponse::CodeBlock { response },
                    });
                }
                MarkdownNode::Block(BlockNode::ListItem(ListItem::Bulleted, _level)) => {
                    ui.horizontal(|ui| {
                        let marker_response = self.bullet(ui);
                        let text_response = self.list_item(ui);
                        components.push(Component {
                            ast_node_idx,
                            response: ComponentResponse::ListItem {
                                marker_response,
                                text_response,
                            },
                        });
                    });
                }
                MarkdownNode::Block(BlockNode::ListItem(ListItem::Numbered(_number), _level)) => {
                    ui.horizontal(|ui| {
                        let marker_response = self.bullet(ui);
                        let text_response = self.list_item(ui);
                        components.push(Component {
                            ast_node_idx,
                            response: ComponentResponse::ListItem {
                                marker_response,
                                text_response,
                            },
                        });
                    });
                }
                MarkdownNode::Block(BlockNode::ListItem(ListItem::Todo(_checked), _level)) => {
                    ui.horizontal(|ui| {
                        let marker_response = self.bullet(ui);
                        let text_response = self.list_item(ui);
                        components.push(Component {
                            ast_node_idx,
                            response: ComponentResponse::ListItem {
                                marker_response,
                                text_response,
                            },
                        });
                    });
                }
                MarkdownNode::Block(BlockNode::Rule) => {
                    let response = self.rule(ui);
                    components.push(Component {
                        ast_node_idx,
                        response: ComponentResponse::Rule { response },
                    });
                }
            }
        }

        Components { components }
    }

    fn paragraph(&self, ui: &mut Ui) -> Response {
        let mut layout_job: LayoutJob = Default::default();
        let mut text_format = Default::default();
        RenderStyle::Markdown(MarkdownNode::Paragraph)
            .apply_style(&mut text_format, &self.appearance);
        layout_job.append("paragraph", 0.0, text_format);
        layout_job.wrap.max_width = ui.available_width();
        let galley = ui.ctx().fonts(|f| f.layout_job(layout_job));
        let desired_size = Vec2::new(ui.available_width(), galley.size().y);
        let (_, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        ui.painter()
            .galley(response.rect.min, galley, Color32::TRANSPARENT);

        response
    }

    fn heading(&self, ui: &mut Ui) -> Response {
        let mut layout_job: LayoutJob = Default::default();
        let mut text_format = Default::default();
        RenderStyle::Markdown(MarkdownNode::Block(BlockNode::Heading(HeadingLevel::H1)))
            .apply_style(&mut text_format, &self.appearance);
        layout_job.append("heading", 0.0, text_format);
        layout_job.wrap.max_width = ui.available_width();
        let galley = ui.ctx().fonts(|f| f.layout_job(layout_job));
        let desired_size = Vec2::new(ui.available_width(), galley.size().y);
        let (_, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        ui.painter()
            .galley(response.rect.min, galley, Color32::TRANSPARENT);

        response
    }

    fn quote(&self, ui: &mut Ui) -> Response {
        let mut layout_job: LayoutJob = Default::default();
        let mut text_format = Default::default();
        RenderStyle::Markdown(MarkdownNode::Block(BlockNode::Quote))
            .apply_style(&mut text_format, &self.appearance);
        layout_job.append("quote", 0.0, text_format);
        layout_job.wrap.max_width = ui.available_width();
        let galley = ui.ctx().fonts(|f| f.layout_job(layout_job));
        let desired_size = Vec2::new(ui.available_width(), galley.size().y);
        let (_, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        ui.painter()
            .galley(response.rect.min, galley, Color32::TRANSPARENT);

        response
    }

    fn code_block(&self, ui: &mut Ui) -> Response {
        let mut layout_job: LayoutJob = Default::default();
        let mut text_format = Default::default();
        RenderStyle::Markdown(MarkdownNode::Block(BlockNode::Code))
            .apply_style(&mut text_format, &self.appearance);
        layout_job.append("code block", 0.0, text_format);
        layout_job.wrap.max_width = ui.available_width();
        let galley = ui.ctx().fonts(|f| f.layout_job(layout_job));
        let desired_size = Vec2::new(ui.available_width(), galley.size().y);
        let (_, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        ui.painter()
            .galley(response.rect.min, galley, Color32::TRANSPARENT);

        response
    }

    fn list_item(&self, ui: &mut Ui) -> Response {
        let mut layout_job: LayoutJob = Default::default();
        let mut text_format = Default::default();
        RenderStyle::Markdown(MarkdownNode::Block(BlockNode::ListItem(ListItem::Bulleted, 0)))
            .apply_style(&mut text_format, &self.appearance);
        layout_job.append("bulleted list item", 0.0, text_format);
        layout_job.wrap.max_width = ui.available_width();
        let galley = ui.ctx().fonts(|f| f.layout_job(layout_job));
        let desired_size = Vec2::new(ui.available_width(), galley.size().y);
        let (_, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        ui.painter()
            .galley(response.rect.min, galley, Color32::TRANSPARENT);

        response
    }

    fn bullet(&self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::new(10.0, 10.0);
        let (_, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        let center = response.rect.center();
        let radius = 3.0;
        ui.painter()
            .circle_filled(center, radius, self.appearance.text());

        response
    }

    fn rule(&self, ui: &mut Ui) -> Response {
        let desired_size = Vec2::new(ui.available_width(), 10.0);
        let (_, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

        let y = response.rect.min.y + response.rect.height() / 2.0;
        let min = Pos2 { x: response.rect.min.x, y };
        let max = Pos2 { x: response.rect.max.x, y };

        ui.painter()
            .line_segment([min, max], Stroke::new(0.3, self.appearance.rule()));

        response
    }
}
