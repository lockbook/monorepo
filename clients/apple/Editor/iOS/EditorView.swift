import Foundation
import SwiftUI
import SwiftEditor

struct EditorView: UIViewRepresentable {
    
    
    func makeUIView(context: Context) -> UIView {
        let textView = iOSMTK()
        textView.becomeFirstResponder()
        return textView
    }
    
    func makeCoordinator() -> Coordinator {
        Coordinator()
    }
    
    func updateUIView(_ uiView: UIView, context: Context) {
        
    }
}

class Coordinator: NSObject {
    
    override init() {
    }
}
