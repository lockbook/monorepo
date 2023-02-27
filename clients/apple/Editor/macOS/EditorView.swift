import Foundation
import SwiftUI
import SwiftEditor

struct EditorView: View {
    
    @FocusState var focused: Bool
    
    var body: some View {
        w(textLoader: DI.documentLoader)
            .focused($focused)
            .onAppear {
                focused = true
            }
    }
}
