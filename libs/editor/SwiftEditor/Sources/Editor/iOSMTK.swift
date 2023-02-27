#if os(iOS)
import UIKit
import MetalKit

public class iOSMTK: MTKView, UITextInput, UITextInputTokenizer {
    public func rangeEnclosingPosition(_ position: UITextPosition, with granularity: UITextGranularity, inDirection direction: UITextDirection) -> UITextRange? {
        nil
    }
    
    public func isPosition(_ position: UITextPosition, atBoundary granularity: UITextGranularity, inDirection direction: UITextDirection) -> Bool {
        false
    }
    
    public func position(from position: UITextPosition, toBoundary granularity: UITextGranularity, inDirection direction: UITextDirection) -> UITextPosition? {
        nil
    }
    
    public func isPosition(_ position: UITextPosition, withinTextUnit granularity: UITextGranularity, inDirection direction: UITextDirection) -> Bool {
        false
    }
    
    public func text(in range: UITextRange) -> String? {
        nil
    }
    
    public func replace(_ range: UITextRange, withText text: String) {
        
    }
    
    public var selectedTextRange: UITextRange?
    
    public var markedTextRange: UITextRange?
    
    public var markedTextStyle: [NSAttributedString.Key : Any]?
    
    public func setMarkedText(_ markedText: String?, selectedRange: NSRange) {
        
    }
    
    public func unmarkText() {
        
    }
    
    public var beginningOfDocument: UITextPosition = UITextPosition.init()
    
    public var endOfDocument: UITextPosition = UITextPosition.init()
    
    public func textRange(from fromPosition: UITextPosition, to toPosition: UITextPosition) -> UITextRange? {
        nil
    }
    
    public func position(from position: UITextPosition, offset: Int) -> UITextPosition? {
        nil
    }
    
    public func position(from position: UITextPosition, in direction: UITextLayoutDirection, offset: Int) -> UITextPosition? {
        nil
        
    }
    
    public func compare(_ position: UITextPosition, to other: UITextPosition) -> ComparisonResult {
        ComparisonResult.orderedAscending
    }
    
    public func offset(from: UITextPosition, to toPosition: UITextPosition) -> Int {
        0
    }
    
    public var inputDelegate: UITextInputDelegate?
    
    public var tokenizer: UITextInputTokenizer  {
        self
    }
    
    public func position(within range: UITextRange, farthestIn direction: UITextLayoutDirection) -> UITextPosition? {
        nil
    }
    
    public func characterRange(byExtending position: UITextPosition, in direction: UITextLayoutDirection) -> UITextRange? {
        nil
    }
    
    public func baseWritingDirection(for position: UITextPosition, in direction: UITextStorageDirection) -> NSWritingDirection {
        NSWritingDirection.natural
    }
    
    public func setBaseWritingDirection(_ writingDirection: NSWritingDirection, for range: UITextRange) {
        
    }
    
    public func firstRect(for range: UITextRange) -> CGRect {
        CGRect.zero
    }
    
    public func caretRect(for position: UITextPosition) -> CGRect {
        CGRect.zero
    }
    
    public func selectionRects(for range: UITextRange) -> [UITextSelectionRect] {
        []
    }
    
    public func closestPosition(to point: CGPoint) -> UITextPosition? {
        nil
    }
    
    public func closestPosition(to point: CGPoint, within range: UITextRange) -> UITextPosition? {
        nil
    }
    
    public func characterRange(at point: CGPoint) -> UITextRange? {
        nil
    }
    
    public var hasText: Bool = false
    
    public func insertText(_ text: String) {
        print(text)
    }
    
    public func deleteBackward() {
        print("back")
    }
    
    public override var canBecomeFirstResponder: Bool {
        print("was asked")
        return true
    }
    
    override init(frame frameRect: CGRect, device: MTLDevice?) {
        super.init(frame: frameRect, device: device)
        self.isPaused = true
        self.enableSetNeedsDisplay = true
    }
    
    required init(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
    
}

#endif
