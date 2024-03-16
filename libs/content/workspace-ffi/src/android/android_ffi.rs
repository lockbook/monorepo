use crate::android::window;
use crate::android::window::NativeWindow;
// use crate::input::canonical::{Location, Modification, Region};
// use crate::input::cursor::Cursor;
// use crate::offset_types::DocCharOffset;
// use crate::style::{BlockNode, InlineNode, ListItem, MarkdownNode};
use crate::{wgpu, CompositeAlphaMode, WgpuWorkspace};
use egui::{
    Context, Event, FontDefinitions, PointerButton, Pos2, TouchDeviceId, TouchId, TouchPhase,
};
use egui_wgpu_backend::ScreenDescriptor;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jfloat, jint, jlong, jobject, jstring};
use jni::JNIEnv;
use lb_external_interface::lb_rs::Uuid;
use lb_external_interface::Core;
use std::time::Instant;
use workspace_rs::register_fonts;
use workspace_rs::theme::visuals;
use workspace_rs::workspace::{Workspace, WsConfig};

use super::keyboard::AndroidKeys;

// I REMOVED MUT FROM ENV
#[no_mangle]
pub extern "system" fn Java_app_lockbook_workspace_Workspace_createWgpuCanvas(
    env: JNIEnv, _: JClass, surface: jobject, core: jlong, scale_factor: jfloat, dark_mode: bool,
    workspace: jlong,
) -> jlong {
    let core = unsafe { &mut *(core as *mut Core) };
    let writable_dir = core.get_config().unwrap().writeable_path;

    let ws_cfg = WsConfig { data_dir: writable_dir, ..Default::default() };

    let context = Context::default();
    visuals::init(&context, dark_mode);
    let mut fonts = FontDefinitions::default();
    register_fonts(&mut fonts);
    context.set_fonts(fonts);

    let mut ws = Workspace::new(ws_cfg, core, &context);
    if workspace != jlong::MAX {
        let old_ws = unsafe { Box::from_raw(workspace as *mut Workspace) };

        ws.active_tab = old_ws.active_tab;
        ws.tabs = old_ws.tabs;
        ws.pers_status = old_ws.pers_status;
    }

    let native_window = NativeWindow::new(&env, surface);
    let backends = wgpu::Backends::VULKAN;
    let instance_desc = wgpu::InstanceDescriptor { backends, ..Default::default() };
    let instance = wgpu::Instance::new(instance_desc);
    let surface = unsafe { instance.create_surface(&native_window).unwrap() };
    let (adapter, device, queue) =
        pollster::block_on(window::request_device(&instance, backends, &surface));
    let format = surface.get_capabilities(&adapter).formats[0];
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: native_window.get_width(),
        height: native_window.get_height(),
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode: CompositeAlphaMode::Auto,
        view_formats: vec![],
    };
    surface.configure(&device, &config);
    let rpass = egui_wgpu_backend::RenderPass::new(&device, format, 1);

    let start_time = Instant::now();
    let obj = WgpuWorkspace {
        start_time,
        device,
        queue,
        surface,
        adapter,
        rpass,
        screen: ScreenDescriptor {
            physical_width: native_window.get_width(),
            physical_height: native_window.get_height(),
            scale_factor,
        },
        context: context.clone(),
        raw_input: Default::default(),
        workspace: ws,
        surface_width: 0,
        surface_height: 0,
    };

    Box::into_raw(Box::new(obj)) as jlong
}

#[no_mangle]
pub extern "system" fn Java_app_lockbook_workspace_Workspace_enterFrame(
    env: JNIEnv, _: JClass, obj: jlong,
) -> jstring {
    let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

    env.new_string(serde_json::to_string(&obj.frame()).unwrap())
        .expect("Couldn't create JString from rust string!")
        .into_raw()
}

#[no_mangle]
pub extern "system" fn Java_app_lockbook_workspace_Workspace_resizeEditor(
    env: JNIEnv, _: JClass, obj: jlong, surface: jobject, scale_factor: jfloat,
) {
    let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };
    let native_window = NativeWindow::new(&env, surface);

    obj.screen.physical_width = native_window.get_width();
    obj.screen.physical_height = native_window.get_height();
    obj.screen.scale_factor = scale_factor;
}

#[no_mangle]
pub extern "system" fn Java_app_lockbook_workspace_Workspace_dropWgpuCanvas(
    mut _env: JNIEnv, _: JClass, obj: jlong,
) -> jlong {
    let obj: Box<WgpuWorkspace> = unsafe { Box::from_raw(obj as *mut _) };

    return Box::into_raw(Box::new(obj.workspace)) as jlong;
}

#[no_mangle]
pub extern "system" fn Java_app_lockbook_workspace_Workspace_sendKeyEvent(
    mut env: JNIEnv, _: JClass, obj: jlong, key_code: jint, content: JString, pressed: jboolean,
    alt: jboolean, ctrl: jboolean, shift: jboolean,
) {
    let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

    let modifiers = egui::Modifiers {
        alt: alt == 1,
        ctrl: ctrl == 1,
        shift: shift == 1,
        mac_cmd: false,
        command: false,
    };

    obj.raw_input.modifiers = modifiers;

    let Some(key) = AndroidKeys::from(key_code) else { return };

    if pressed == 1 && (modifiers.shift_only() || modifiers.is_none()) && key.valid_text() {
        let text: String = match env.get_string(&content) {
            Ok(cont) => cont.into(),
            Err(err) => format!("# The error is: {:?}", err),
        };

        obj.raw_input.events.push(Event::Text(text));
    }

    if let Some(key) = key.egui_key() {
        obj.raw_input.events.push(Event::Key {
            key,
            pressed: pressed == 1,
            repeat: false,
            modifiers,
        });
    } else {
    }
}

#[no_mangle]
pub extern "system" fn Java_app_lockbook_workspace_Workspace_touchesBegin(
    _env: JNIEnv, _: JClass, obj: jlong, id: jint, x: jfloat, y: jfloat, pressure: jfloat,
) {
    let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

    println!("registering on begin: ({}, {})", x, y);

    obj.raw_input.events.push(Event::Touch {
        device_id: TouchDeviceId(0),
        id: TouchId(id as u64),
        phase: TouchPhase::Start,
        pos: Pos2 { x, y },
        force: pressure,
    });

    obj.raw_input.events.push(Event::PointerButton {
        pos: Pos2 { x, y },
        button: PointerButton::Primary,
        pressed: true,
        modifiers: Default::default(),
    });
}

#[no_mangle]
pub extern "system" fn Java_app_lockbook_workspace_Workspace_touchesMoved(
    _env: JNIEnv, _: JClass, obj: jlong, id: jint, x: jfloat, y: jfloat, pressure: jfloat,
) {
    let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

    println!("registering on moved: ({}, {})", x, y);

    obj.raw_input.events.push(Event::Touch {
        device_id: TouchDeviceId(0),
        id: TouchId(id as u64),
        phase: TouchPhase::Move,
        pos: Pos2 { x, y },
        force: pressure,
    });

    obj.raw_input
        .events
        .push(Event::PointerMoved(Pos2 { x, y }));
}

#[no_mangle]
pub extern "system" fn Java_app_lockbook_workspace_Workspace_touchesEnded(
    _env: JNIEnv, _: JClass, obj: jlong, id: jint, x: jfloat, y: jfloat, pressure: jfloat,
) {
    let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

    println!("registering on ended: ({}, {})", x, y);

    obj.raw_input.events.push(Event::Touch {
        device_id: TouchDeviceId(0),
        id: TouchId(id as u64),
        phase: TouchPhase::End,
        pos: Pos2 { x, y },
        force: pressure,
    });

    obj.raw_input.events.push(Event::PointerButton {
        pos: Pos2 { x, y },
        button: PointerButton::Primary,
        pressed: false,
        modifiers: Default::default(),
    });

    obj.raw_input.events.push(Event::PointerGone);
}

#[no_mangle]
pub extern "system" fn Java_app_lockbook_workspace_Workspace_openFile(
    mut env: JNIEnv, _: JClass, obj: jlong, jid: JString, new_file: jboolean,
) {
    let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

    let rid: String = env.get_string(&jid).unwrap().into();
    let id = Uuid::parse_str(&rid).unwrap();

    obj.workspace.open_file(id, new_file == 1);
}

// let id = id.into();

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_getAllText(
//     env: JNIEnv, _: JClass, obj: jlong,
// ) -> jstring {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     env.new_string(&obj.workspace.buffer.current.text)
//         .expect("Couldn't create JString from rust string!")
//         .into_raw()
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_getSelection(
//     env: JNIEnv, _: JClass, obj: jlong,
// ) -> jstring {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     let (start, end) = obj.workspace.buffer.current.cursor.selection;
//     let selection_text = format!("{} {}", start.0, end.0);

//     env.new_string(selection_text)
//         .expect("Couldn't create JString from rust string!")
//         .into_raw()
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_setSelection(
//     _env: JNIEnv, _: JClass, obj: jlong, start: jint, end: jint,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     obj.workspace.custom_events.push(Modification::Select {
//         region: Region::BetweenLocations {
//             start: Location::DocCharOffset(DocCharOffset(start as usize)),
//             end: Location::DocCharOffset(DocCharOffset(end as usize)),
//         },
//     });
// }

// // editable stuff

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_getTextLength(
//     _env: JNIEnv, _: JClass, obj: jlong,
// ) -> jint {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     return obj.workspace.buffer.current.segs.last_cursor_position().0 as jint;
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_clear(
//     _env: JNIEnv, _: JClass, obj: jlong,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     obj.workspace.custom_events.push(Modification::Replace {
//         region: Region::BetweenLocations {
//             start: Location::DocCharOffset(DocCharOffset(0)),
//             end: Location::DocCharOffset(obj.workspace.buffer.current.segs.last_cursor_position()),
//         },
//         text: "".to_string(),
//     })
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_replace(
//     mut env: JNIEnv, _: JClass, obj: jlong, start: jint, end: jint, text: JString,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     let text: String = match env.get_string(&text) {
//         Ok(cont) => cont.into(),
//         Err(err) => format!("error: {:?}", err),
//     };

//     obj.workspace.custom_events.push(Modification::Replace {
//         region: Region::BetweenLocations {
//             start: Location::DocCharOffset(DocCharOffset(start as usize)),
//             end: Location::DocCharOffset(DocCharOffset(end as usize)),
//         },
//         text,
//     })
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_insert(
//     mut env: JNIEnv, _: JClass, obj: jlong, index: jint, text: JString,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     let text: String = match env.get_string(&text) {
//         Ok(cont) => cont.into(),
//         Err(err) => format!("error: {:?}", err),
//     };

//     let loc = Location::DocCharOffset(DocCharOffset(index as usize));

//     obj.workspace.custom_events.push(Modification::Replace {
//         region: Region::BetweenLocations { start: loc, end: loc },
//         text,
//     })
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_append(
//     mut env: JNIEnv, _: JClass, obj: jlong, text: JString,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     let text: String = match env.get_string(&text) {
//         Ok(cont) => cont.into(),
//         Err(err) => format!("error: {:?}", err),
//     };

//     let loc = Location::DocCharOffset(obj.workspace.buffer.current.segs.last_cursor_position());

//     obj.workspace.custom_events.push(Modification::Replace {
//         region: Region::BetweenLocations { start: loc, end: loc },
//         text,
//     })
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_getTextInRange(
//     env: JNIEnv, _: JClass, obj: jlong, start: jint, end: jint,
// ) -> jstring {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     let cursor: Cursor = (start as usize, end as usize).into();

//     let buffer = &obj.workspace.buffer.current;
//     let text = cursor.selection_text(buffer);

//     env.new_string(text)
//         .expect("Couldn't create JString from rust string!")
//         .into_raw()
// }

// #[derive(Serialize)]
// pub struct AndroidRect {
//     min_x: f32,
//     min_y: f32,
//     max_x: f32,
//     max_y: f32,
// }

// // context menu

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_selectAll(
//     _env: JNIEnv, _: JClass, obj: jlong,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     let buffer = &obj.workspace.buffer.current;

//     obj.workspace.custom_events.push(Modification::Select {
//         region: Region::BetweenLocations {
//             start: Location::DocCharOffset(DocCharOffset(0)),
//             end: Location::DocCharOffset(buffer.segs.last_cursor_position()),
//         },
//     });
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_clipboardChanged(
//     mut env: JNIEnv, _: JClass, obj: jlong, text: JString,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     let content: String = match env.get_string(&text) {
//         Ok(cont) => cont.into(),
//         Err(_) => "didn't work?".to_string(),
//     };

//     obj.from_host = Some(content);
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_hasCopiedText(
//     _env: JNIEnv, _: JClass, obj: jlong,
// ) -> jboolean {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     if obj.from_egui.is_some() {
//         1
//     } else {
//         0
//     }
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_getCopiedText(
//     env: JNIEnv, _: JClass, obj: jlong,
// ) -> jstring {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     let copied_text = obj.from_egui.take().unwrap_or_default();

//     env.new_string(copied_text)
//         .expect("Couldn't create JString from rust string!")
//         .into_raw()
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_clipboardCut(
//     _env: JNIEnv, _: JClass, obj: jlong,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };
//     obj.workspace.custom_events.push(Modification::Cut);
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_clipboardCopy(
//     _env: JNIEnv, _: JClass, obj: jlong,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };
//     obj.workspace.custom_events.push(Modification::Copy);
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_clipboardPaste(
//     _env: JNIEnv, _: JClass, obj: jlong,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     let clip = obj.from_host.clone().unwrap_or_default();
//     obj.raw_input.events.push(Event::Paste(clip));
// }

// // markdown syntax insert

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_applyStyleToSelectionHeading(
//     _env: JNIEnv, _: JClass, obj: jlong, heading_size: jint,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     obj.workspace
//         .custom_events
//         .push(Modification::toggle_heading_style(heading_size as usize));
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_applyStyleToSelectionBulletedList(
//     _env: JNIEnv, _: JClass, obj: jlong,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     obj.workspace
//         .custom_events
//         .push(Modification::toggle_block_style(BlockNode::ListItem(ListItem::Bulleted, 0)));
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_applyStyleToSelectionNumberedList(
//     _env: JNIEnv, _: JClass, obj: jlong,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     obj.workspace
//         .custom_events
//         .push(Modification::toggle_block_style(BlockNode::ListItem(ListItem::Numbered(1), 0)));
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_applyStyleToSelectionTodoList(
//     _env: JNIEnv, _: JClass, obj: jlong,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     obj.workspace
//         .custom_events
//         .push(Modification::toggle_block_style(BlockNode::ListItem(ListItem::Todo(false), 0)));
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_applyStyleToSelectionBold(
//     _env: JNIEnv, _: JClass, obj: jlong,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     obj.workspace.custom_events.push(Modification::ToggleStyle {
//         region: Region::Selection,
//         style: MarkdownNode::Inline(InlineNode::Bold),
//     });
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_applyStyleToSelectionItalic(
//     _env: JNIEnv, _: JClass, obj: jlong,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     obj.workspace.custom_events.push(Modification::ToggleStyle {
//         region: Region::Selection,
//         style: MarkdownNode::Inline(InlineNode::Italic),
//     });
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_applyStyleToSelectionInlineCode(
//     _env: JNIEnv, _: JClass, obj: jlong,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     obj.workspace.custom_events.push(Modification::ToggleStyle {
//         region: Region::Selection,
//         style: MarkdownNode::Inline(InlineNode::Code),
//     });
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_applyStyleToSelectionStrikethrough(
//     _env: JNIEnv, _: JClass, obj: jlong,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     obj.workspace.custom_events.push(Modification::ToggleStyle {
//         region: Region::Selection,
//         style: MarkdownNode::Inline(InlineNode::Strikethrough),
//     });
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_indentAtCursor(
//     _env: JNIEnv, _: JClass, obj: jlong, deindent: jboolean,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     obj.workspace
//         .custom_events
//         .push(Modification::Indent { deindent: deindent == 1 });
// }

// #[no_mangle]
// pub extern "system" fn Java_app_lockbook_egui_1editor_EGUIEditor_undoRedo(
//     _env: JNIEnv, _: JClass, obj: jlong, redo: jboolean,
// ) {
//     let obj = unsafe { &mut *(obj as *mut WgpuWorkspace) };

//     if redo == 1 {
//         obj.workspace.custom_events.push(Modification::Redo);
//     } else {
//         obj.workspace.custom_events.push(Modification::Undo);
//     }
// }
