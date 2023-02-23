import Foundation
import SwiftUI

struct EditorView: UIViewRepresentable {

    @EnvironmentObject var model: DocumentLoader
    let frame: CGRect

    func makeUIView(context: Context) -> UIView {

        let textView = CustomUITextView()
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
