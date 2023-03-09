use crate::{CTextPosition, CTextRange, CustomEvents, WgpuEditor};
use egui::{Event, Key};
use std::ffi::{c_char, c_void, CStr};

/// # Safety
/// https://developer.apple.com/documentation/uikit/uikeyinput/1614543-inserttext
#[no_mangle]
pub unsafe extern "C" fn insert_text(obj: *mut c_void, content: *const c_char) {
    let obj = &mut *(obj as *mut WgpuEditor);
    let content = CStr::from_ptr(content).to_str().unwrap().into();
    obj.raw_input.events.push(Event::Text(content))
}

/// # Safety
/// https://developer.apple.com/documentation/uikit/uikeyinput/1614543-inserttext
#[no_mangle]
pub unsafe extern "C" fn backspace(obj: *mut c_void) {
    let obj = &mut *(obj as *mut WgpuEditor);
    obj.raw_input.events.push(Event::Key {
        key: Key::Backspace,
        pressed: true,
        modifiers: Default::default(),
    });
}

/// # Safety
/// https://developer.apple.com/documentation/uikit/uikeyinput/1614457-hastext
#[no_mangle]
pub unsafe extern "C" fn has_text(obj: *mut c_void) -> bool {
    let obj = &mut *(obj as *mut WgpuEditor);
    !obj.editor.buffer.is_empty()
}

/// # Safety
/// https://developer.apple.com/documentation/uikit/uitextinput/1614558-replace
#[no_mangle]
pub unsafe extern "C" fn replace_text(obj: *mut c_void, range: CTextRange, text: *const c_char) {
    let obj = &mut *(obj as *mut WgpuEditor);
    let text = CStr::from_ptr(text).to_str().unwrap().into();
    obj.editor
        .events
        .push(CustomEvents::ReplaceText(text, range))
}

/// # Safety: obj must be a valid pointer to WgpuEditor
/// https://developer.apple.com/documentation/uikit/uitextinput/1614527-text
#[no_mangle]
pub unsafe extern "C" fn text_in_range(obj: *mut c_void, range: CTextRange) -> *const c_char {
    let obj = &mut *(obj as *mut WgpuEditor);
    todo!("travis")
}

/// # Safety: obj must be a valid pointer to WgpuEditor
/// https://developer.apple.com/documentation/uikit/uitextinput/1614541-selectedtextrange
#[no_mangle]
pub unsafe extern "C" fn get_selected(obj: *mut c_void) -> CTextRange {
    todo!("travis")
}

/// # Safety: obj must be a valid pointer to WgpuEditor
/// https://developer.apple.com/documentation/uikit/uitextinput/1614541-selectedtextrange
#[no_mangle]
pub unsafe extern "C" fn set_selected(obj: *mut c_void, range: CTextRange) {
    let obj = &mut *(obj as *mut WgpuEditor);
    obj.editor.events.push(CustomEvents::SetSelected(range))
}

/// # Safety: obj must be a valid pointer to WgpuEditor
/// https://developer.apple.com/documentation/uikit/uitextinput/1614489-markedtextrange
#[no_mangle]
pub unsafe extern "C" fn get_marked(obj: *mut c_void) -> CTextRange {
    todo!("travis")
}

/// # Safety: obj must be a valid pointer to WgpuEditor
/// https://developer.apple.com/documentation/uikit/uitextinput/1614465-setmarkedtext
#[no_mangle]
pub unsafe extern "C" fn set_marked(obj: *mut c_void, range: CTextRange, text: *const c_char) {
    let obj = &mut *(obj as *mut WgpuEditor);
    let text = CStr::from_ptr(text).to_str().unwrap().into();
    obj.editor.events.push(CustomEvents::SetMarked(text, range))
}

/// https://developer.apple.com/documentation/uikit/uitextinput/1614512-unmarktext
#[no_mangle]
pub unsafe extern "C" fn unmark_text(obj: *mut c_void) {
    let obj = &mut *(obj as *mut WgpuEditor);
    obj.editor.events.push(CustomEvents::UnmarkText);
}

/// # Safety: obj must be a valid pointer to WgpuEditor
/// https://developer.apple.com/documentation/uikit/uitextinput/1614489-markedtextrange
/// isn't this always just going to be 0?
/// should we be returning a subset of the document? https://stackoverflow.com/questions/12676851/uitextinput-is-it-ok-to-return-incorrect-beginningofdocument-endofdocumen
#[no_mangle]
pub unsafe extern "C" fn beginning_of_document(_obj: *mut c_void) -> CTextPosition {
    CTextPosition { pos: 0 }
}

/// # Safety: obj must be a valid pointer to WgpuEditor
/// https://developer.apple.com/documentation/uikit/uitextinput/1614489-markedtextrange
/// should we be returning a subset of the document? https://stackoverflow.com/questions/12676851/uitextinput-is-it-ok-to-return-incorrect-beginningofdocument-endofdocumen
#[no_mangle]
pub unsafe extern "C" fn end_of_document(obj: *mut c_void) -> CTextPosition {
    let obj = &mut *(obj as *mut WgpuEditor);
    CTextPosition { pos: obj.editor.buffer.current.segs.grapheme_indexes.len() }
}
