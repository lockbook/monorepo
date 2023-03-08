use crate::{CustomEvents, UITextRange, WgpuEditor};
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
pub unsafe extern "C" fn replace_text(obj: *mut c_void, range: UITextRange, text: *const c_char) {
    let obj = &mut *(obj as *mut WgpuEditor);
    let text = CStr::from_ptr(text).to_str().unwrap().into();
    obj.editor
        .events
        .push(CustomEvents::ReplaceText(text, range))
}

/// https://developer.apple.com/documentation/uikit/uitextinput/1614527-text
pub unsafe extern "C" fn text_in_range(obj: *mut c_void, range: UITextRange) -> *const c_char {
    todo!("travis")
}

/// https://developer.apple.com/documentation/uikit/uitextinput/1614541-selectedtextrange
pub unsafe extern "C" fn get_selected(obj: *mut c_void) -> UITextRange {
    todo!("travis")
}

/// https://developer.apple.com/documentation/uikit/uitextinput/1614541-selectedtextrange
pub unsafe extern "C" fn set_selected(obj: *mut c_void, range: UITextRange) {
    let obj = &mut *(obj as *mut WgpuEditor);
    obj.editor.events.push(CustomEvents::SetSelected(range))
}

/// https://developer.apple.com/documentation/uikit/uitextinput/1614489-markedtextrange
pub unsafe extern "C" fn get_marked(obj: *mut c_void) -> UITextRange {
    todo!("travis")
}

/// https://developer.apple.com/documentation/uikit/uitextinput/1614465-setmarkedtext
pub unsafe extern "C" fn set_marked(obj: *mut c_void, range: UITextRange, text: *const c_char) {
    let obj = &mut *(obj as *mut WgpuEditor);
    let text = CStr::from_ptr(text).to_str().unwrap().into();
    obj.editor.events.push(CustomEvents::SetMarked(text, range))
}

/// https://developer.apple.com/documentation/uikit/uitextinput/1614512-unmarktext
pub unsafe extern "C" fn unmark_text(obj: *mut c_void) {
    let obj = &mut *(obj as *mut WgpuEditor);
    obj.editor.events.push(CustomEvents::UnmarkText);
}
