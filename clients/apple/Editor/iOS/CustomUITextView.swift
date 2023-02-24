import UIKit
import SwiftUI

class CustomUITextView: UIView, UITextInput, UITextInputTokenizer {
    
    func rangeEnclosingPosition(_ position: UITextPosition, with granularity: UITextGranularity, inDirection direction: UITextDirection) -> UITextRange? {
        nil
    }
    
    func isPosition(_ position: UITextPosition, atBoundary granularity: UITextGranularity, inDirection direction: UITextDirection) -> Bool {
        false
    }
    
    func position(from position: UITextPosition, toBoundary granularity: UITextGranularity, inDirection direction: UITextDirection) -> UITextPosition? {
        nil
    }
    
    func isPosition(_ position: UITextPosition, withinTextUnit granularity: UITextGranularity, inDirection direction: UITextDirection) -> Bool {
        false
    }
    
    func text(in range: UITextRange) -> String? {
        nil
    }
    
    func replace(_ range: UITextRange, withText text: String) {
        
    }
    
    var selectedTextRange: UITextRange?
    
    var markedTextRange: UITextRange?
    
    var markedTextStyle: [NSAttributedString.Key : Any]?
    
    func setMarkedText(_ markedText: String?, selectedRange: NSRange) {
        
    }
    
    func unmarkText() {
        
    }
    
    var beginningOfDocument: UITextPosition = UITextPosition.init()
    
    var endOfDocument: UITextPosition = UITextPosition.init()
    
    func textRange(from fromPosition: UITextPosition, to toPosition: UITextPosition) -> UITextRange? {
        nil
    }
    
    func position(from position: UITextPosition, offset: Int) -> UITextPosition? {
        nil
    }
    
    func position(from position: UITextPosition, in direction: UITextLayoutDirection, offset: Int) -> UITextPosition? {
        nil
        
    }
    
    func compare(_ position: UITextPosition, to other: UITextPosition) -> ComparisonResult {
        ComparisonResult.orderedAscending
    }
    
    func offset(from: UITextPosition, to toPosition: UITextPosition) -> Int {
        0
    }
    
    var inputDelegate: UITextInputDelegate?
    
    var tokenizer: UITextInputTokenizer  {
        self
    }
    
    func position(within range: UITextRange, farthestIn direction: UITextLayoutDirection) -> UITextPosition? {
        nil
    }
    
    func characterRange(byExtending position: UITextPosition, in direction: UITextLayoutDirection) -> UITextRange? {
        nil
    }
    
    func baseWritingDirection(for position: UITextPosition, in direction: UITextStorageDirection) -> NSWritingDirection {
        NSWritingDirection.natural
    }
    
    func setBaseWritingDirection(_ writingDirection: NSWritingDirection, for range: UITextRange) {
        
    }
    
    func firstRect(for range: UITextRange) -> CGRect {
        CGRect.zero
    }
    
    func caretRect(for position: UITextPosition) -> CGRect {
        CGRect.zero
    }
    
    func selectionRects(for range: UITextRange) -> [UITextSelectionRect] {
        []
    }
    
    func closestPosition(to point: CGPoint) -> UITextPosition? {
        nil
    }
    
    func closestPosition(to point: CGPoint, within range: UITextRange) -> UITextPosition? {
        nil
    }
    
    func characterRange(at point: CGPoint) -> UITextRange? {
        nil
    }
    
    var hasText: Bool = false
    
    func insertText(_ text: String) {
        print(text)
    }
    
    func deleteBackward() {
        print("back")
    }
    
    override var canBecomeFirstResponder: Bool {
        print("was asked")
        return true
    }
    
    
    // 1
    private var label: UILabel = {
        let label = UILabel()
        label.translatesAutoresizingMaskIntoConstraints = false
        label.font = UIFont.preferredFont(forTextStyle: .title1)
        label.text = "Hello, UIKit!"
        label.textAlignment = .center
        
        return label
    }()
    
    init() {
        
        super.init(frame: .zero)
        // 2
        backgroundColor = .systemPink
        
        
        
        // 3
        addSubview(label)
        NSLayoutConstraint.activate([
            label.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 16),
            label.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -16),
            label.topAnchor.constraint(equalTo: topAnchor, constant: 20),
            label.bottomAnchor.constraint(equalTo: bottomAnchor, constant: -20),
        ])
        
    }
    
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
}
