use crate::WgpuEditor;
use egui::Event;
use std::ffi::{c_char, c_void, CStr};

/// # Safety
#[no_mangle]
pub unsafe extern "C" fn insert_text(obj: *mut c_void, content: *const c_char) {
    let obj = &mut *(obj as *mut WgpuEditor);
    let content = CStr::from_ptr(content).to_str().unwrap().into();
    obj.raw_input.events.push(Event::Text(content))
}
