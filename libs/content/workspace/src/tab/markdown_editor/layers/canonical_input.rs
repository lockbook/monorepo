use crate::tab::{
    self,
    markdown_editor::{
        utils::{
            self,
            style::{BlockNode, InlineNode, ListItem, MarkdownNode},
            Bound, Event, Increment, Offset, Region,
        },
        Editor,
    },
    ClipContent, ExtendedInput as _,
};
use egui::{self, Context, EventFilter, Key, Modifiers};

#[derive(Default)]
pub struct CanonicalInput {
    pub input: Vec<Event>,

    pub frame_nr: u64,

    pub seq: usize,
}

impl Editor {
    pub fn canonical_input(&mut self, ctx: &Context) {
        let Editor { canonical_input, .. } = self;

        if utils::check_assign(&mut canonical_input.frame_nr, ctx.frame_nr()) {
            canonical_input.seq += 1;
        } else {
            return;
        }

        canonical_input.input.clear();
        // canonical_input.input.extend(self.get_cursor_fix_events());
        canonical_input.input.extend(self.get_workspace_events(ctx));
        canonical_input.input.extend(self.get_key_events(ctx));
    }

    // fn get_cursor_fix_events(&self) -> Vec<Event> {
    //     // if the cursor is in an invalid location, move it to the next valid location
    //     let mut fixed_selection = self.buffer.current.selection;
    //     if let BoundCase::BetweenRanges { range_after, .. } =
    //         fixed_selection.0.bound_case(&self.bounds.text)
    //     {
    //         fixed_selection.0 = range_after.start();
    //     }
    //     if let BoundCase::BetweenRanges { range_after, .. } =
    //         fixed_selection.1.bound_case(&self.bounds.text)
    //     {
    //         fixed_selection.1 = range_after.start();
    //     }

    //     if fixed_selection != self.buffer.current.selection {
    //         vec![Event::Select { region: fixed_selection.into() }]
    //     } else {
    //         vec![]
    //     }
    // }

    fn get_workspace_events(&self, ctx: &Context) -> Vec<Event> {
        let mut result = Vec::new();
        for event in ctx.pop_events() {
            match event {
                crate::Event::Markdown(modification) => result.push(modification),
                crate::Event::Drop { content, .. } | crate::Event::Paste { content, .. } => {
                    for clip in content {
                        match clip {
                            ClipContent::Image(data) => {
                                let file = tab::import_image(&self.core, self.file_id, &data);
                                let rel_path =
                                    tab::core_get_relative_path(&self.core, self.file_id, file.id);
                                let markdown_image_link = format!("![{}]({})", file.name, rel_path);

                                result.push(Event::Replace {
                                    region: Region::Selection, // todo: more thoughtful location
                                    text: markdown_image_link,
                                });
                            }
                            ClipContent::Files(..) => {
                                // todo: support file drop & paste
                                println!("unimplemented: editor file drop & paste");
                            }
                        }
                    }
                }
                crate::Event::PredictedTouch { .. } => {}
            }
        }
        result
    }

    fn get_key_events(&self, ctx: &Context) -> Vec<Event> {
        if self.focused(ctx) {
            ctx.input(|r| {
                r.filtered_events(&EventFilter {
                    tab: true,
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    escape: false,
                })
            })
            .into_iter()
            .filter_map(translate_egui_keyboard_event)
            .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    }
}

impl From<Modifiers> for Offset {
    fn from(modifiers: Modifiers) -> Self {
        let should_jump_line = modifiers.mac_cmd;

        let is_apple = cfg!(target_vendor = "apple");
        let is_apple_alt = is_apple && modifiers.alt;
        let is_non_apple_ctrl = !is_apple && modifiers.ctrl;
        let should_jump_word = is_apple_alt || is_non_apple_ctrl;

        if should_jump_line {
            Offset::To(Bound::Line)
        } else if should_jump_word {
            Offset::Next(Bound::Word)
        } else {
            Offset::Next(Bound::Char)
        }
    }
}

/// Translates UI events into editor events. Editor events are interpreted based on the state of the buffer when
/// they're applied, so this translation makes no use of the editor's current state.
pub fn translate_egui_keyboard_event(event: egui::Event) -> Option<Event> {
    match event {
        egui::Event::Key { key, pressed: true, modifiers, .. }
            if matches!(key, Key::ArrowUp | Key::ArrowDown) && !cfg!(target_os = "ios") =>
        {
            Some(Event::Select {
                region: Region::ToOffset {
                    offset: if modifiers.mac_cmd {
                        Offset::To(Bound::Doc)
                    } else {
                        Offset::By(Increment::Line)
                    },
                    backwards: key == Key::ArrowUp,
                    extend_selection: modifiers.shift,
                },
            })
        }
        egui::Event::Key { key, pressed: true, modifiers, .. }
            if matches!(key, Key::ArrowRight | Key::ArrowLeft | Key::Home | Key::End)
                && !cfg!(target_os = "ios") =>
        {
            Some(Event::Select {
                region: Region::ToOffset {
                    offset: if matches!(key, Key::Home | Key::End) {
                        if modifiers.command {
                            Offset::To(Bound::Doc)
                        } else {
                            Offset::To(Bound::Line)
                        }
                    } else {
                        Offset::from(modifiers)
                    },
                    backwards: matches!(key, Key::ArrowLeft | Key::Home),
                    extend_selection: modifiers.shift,
                },
            })
        }
        egui::Event::Text(text) | egui::Event::Paste(text) => {
            Some(Event::Replace { region: Region::Selection, text: text.clone() })
        }
        egui::Event::Key { key, pressed: true, modifiers, .. }
            if matches!(key, Key::Backspace | Key::Delete) =>
        {
            Some(Event::Delete {
                region: Region::SelectionOrOffset {
                    offset: Offset::from(modifiers),
                    backwards: key == Key::Backspace,
                },
            })
        }
        egui::Event::Key { key: Key::Enter, pressed: true, modifiers, .. }
            if !cfg!(target_os = "ios") =>
        {
            Some(Event::Newline { advance_cursor: !modifiers.shift })
        }
        egui::Event::Key { key: Key::Tab, pressed: true, modifiers, .. } if !modifiers.alt => {
            if !modifiers.shift && cfg!(target_os = "ios") {
                return None;
            }

            Some(Event::Indent { deindent: modifiers.shift })
        }
        egui::Event::Key { key: Key::A, pressed: true, modifiers, .. }
            if modifiers.command && !cfg!(target_os = "ios") =>
        {
            Some(Event::Select { region: Region::Bound { bound: Bound::Doc, backwards: true } })
        }
        egui::Event::Cut => Some(Event::Cut),
        egui::Event::Key { key: Key::X, pressed: true, modifiers, .. }
            if modifiers.command && !modifiers.shift && !cfg!(target_os = "ios") =>
        {
            Some(Event::Cut)
        }
        egui::Event::Copy => Some(Event::Copy),
        egui::Event::Key { key: Key::C, pressed: true, modifiers, .. }
            if modifiers.command && !modifiers.shift && !cfg!(target_os = "ios") =>
        {
            Some(Event::Copy)
        }
        egui::Event::Key { key: Key::Z, pressed: true, modifiers, .. }
            if modifiers.command && !cfg!(target_os = "ios") =>
        {
            if !modifiers.shift {
                Some(Event::Undo)
            } else {
                Some(Event::Redo)
            }
        }
        egui::Event::Key { key: Key::B, pressed: true, modifiers, .. } if modifiers.command => {
            Some(Event::ToggleStyle {
                region: Region::Selection,
                style: MarkdownNode::Inline(InlineNode::Bold),
            })
        }
        egui::Event::Key { key: Key::I, pressed: true, modifiers, .. } if modifiers.command => {
            Some(Event::ToggleStyle {
                region: Region::Selection,
                style: MarkdownNode::Inline(InlineNode::Italic),
            })
        }
        egui::Event::Key { key: Key::C, pressed: true, modifiers, .. }
            if modifiers.command && modifiers.shift =>
        {
            if !modifiers.alt {
                Some(Event::ToggleStyle {
                    region: Region::Selection,
                    style: MarkdownNode::Inline(InlineNode::Code),
                })
            } else {
                Some(Event::toggle_block_style(BlockNode::Code("".into())))
            }
        }
        egui::Event::Key { key: Key::X, pressed: true, modifiers, .. }
            if modifiers.command && modifiers.shift =>
        {
            Some(Event::ToggleStyle {
                region: Region::Selection,
                style: MarkdownNode::Inline(InlineNode::Strikethrough),
            })
        }
        egui::Event::Key { key: Key::K, pressed: true, modifiers, .. } if modifiers.command => {
            Some(Event::ToggleStyle {
                region: Region::Selection,
                style: MarkdownNode::Inline(InlineNode::Link("".into(), "".into())),
            })
        }
        egui::Event::Key { key: Key::Num7, pressed: true, modifiers, .. }
            if modifiers.command && modifiers.shift =>
        {
            Some(Event::toggle_block_style(BlockNode::ListItem(ListItem::Numbered(1), 0)))
        }
        egui::Event::Key { key: Key::Num8, pressed: true, modifiers, .. }
            if modifiers.command && modifiers.shift =>
        {
            Some(Event::toggle_block_style(BlockNode::ListItem(ListItem::Bulleted, 0)))
        }
        egui::Event::Key { key: Key::Num9, pressed: true, modifiers, .. }
            if modifiers.command && modifiers.shift =>
        {
            Some(Event::toggle_block_style(BlockNode::ListItem(ListItem::Todo(false), 0)))
        }
        egui::Event::Key { key: Key::Num1, pressed: true, modifiers, .. }
            if modifiers.command && modifiers.alt =>
        {
            Some(Event::toggle_block_style(BlockNode::Heading(1)))
        }
        egui::Event::Key { key: Key::Num2, pressed: true, modifiers, .. }
            if modifiers.command && modifiers.alt =>
        {
            Some(Event::toggle_block_style(BlockNode::Heading(2)))
        }
        egui::Event::Key { key: Key::Num3, pressed: true, modifiers, .. }
            if modifiers.command && modifiers.alt =>
        {
            Some(Event::toggle_block_style(BlockNode::Heading(3)))
        }
        egui::Event::Key { key: Key::Num4, pressed: true, modifiers, .. }
            if modifiers.command && modifiers.alt =>
        {
            Some(Event::toggle_block_style(BlockNode::Heading(4)))
        }
        egui::Event::Key { key: Key::Num5, pressed: true, modifiers, .. }
            if modifiers.command && modifiers.alt =>
        {
            Some(Event::toggle_block_style(BlockNode::Heading(5)))
        }
        egui::Event::Key { key: Key::Num6, pressed: true, modifiers, .. }
            if modifiers.command && modifiers.alt =>
        {
            Some(Event::toggle_block_style(BlockNode::Heading(6)))
        }
        egui::Event::Key { key: Key::Q, pressed: true, modifiers, .. }
            if modifiers.command && modifiers.alt =>
        {
            Some(Event::toggle_block_style(BlockNode::Quote))
        }
        egui::Event::Key { key: Key::R, pressed: true, modifiers, .. }
            if modifiers.command && modifiers.alt =>
        {
            Some(Event::toggle_block_style(BlockNode::Rule))
        }
        egui::Event::Key { key: Key::F2, pressed: true, .. } => Some(Event::ToggleDebug),
        egui::Event::Key { key: Key::Equals, pressed: true, modifiers, .. }
            if modifiers.command =>
        {
            Some(Event::IncrementBaseFontSize)
        }
        egui::Event::Key { key: Key::Minus, pressed: true, modifiers, .. } if modifiers.command => {
            Some(Event::DecrementBaseFontSize)
        }
        _ => None,
    }
}

impl Event {
    pub fn toggle_block_style(block: BlockNode) -> Self {
        Event::ToggleStyle {
            region: Region::Bound { bound: Bound::Paragraph, backwards: false },
            style: MarkdownNode::Block(block),
        }
    }
}
