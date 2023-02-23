import UIKit

class CustomUITextView: UIView, UIKeyInput {
    // the string we'll be drawing
    var input = ""

    override var canBecomeFirstResponder: Bool {
        true
    }

    var hasText: Bool {
        input.isEmpty == false
    }

    func insertText(_ text: String) {
        input += text
        setNeedsDisplay()
    }

    func deleteBackward() {
        _ = input.popLast()
        setNeedsDisplay()
    }

    override func draw(_ rect: CGRect) {
        let attrs: [NSAttributedString.Key: Any] = [.font: UIFont.systemFont(ofSize: 32)]
        let attributedString = NSAttributedString(string: input, attributes: attrs)
        attributedString.draw(in: rect)
    }
}
