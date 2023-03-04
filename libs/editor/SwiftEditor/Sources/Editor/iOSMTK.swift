#if os(iOS)
import UIKit
import MetalKit
import Bridge

public class iOSMTK: MTKView, MTKViewDelegate, UITextInput, UITextInputTokenizer {
    var editorHandle: UnsafeMutableRawPointer?
    
    override init(frame frameRect: CGRect, device: MTLDevice?) {
        super.init(frame: frameRect, device: device)
        
        let metalLayer = UnsafeMutableRawPointer(Unmanaged.passRetained(self.layer).toOpaque())
        self.editorHandle = init_editor(metalLayer, "# hello world", false) // todo
        
        self.isPaused = true
        self.enableSetNeedsDisplay = true
        self.delegate = self
    }
    
    public func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {
        resize_editor(editorHandle, Float(size.width), Float(size.height), Float(self.contentScaleFactor))
        self.setNeedsDisplay(self.frame)
    }
    
    public func draw(in view: MTKView) {
        //        dark_mode(editorHandle, (view as! CustomMTK).isDarkMode())
        set_scale(editorHandle, Float(self.contentScaleFactor))
        draw_editor(editorHandle)
    }
    
    public func insertText(_ text: String) {
        print("\(#function)")
        insert_text(editorHandle, text)
        self.setNeedsDisplay(self.frame)
    }
    
    public func text(in range: UITextRange) -> String? {
        print("\(#function)")
        return nil
    }
    
    
    public func replace(_ range: UITextRange, withText text: String) {
        print("\(#function)")
        
    }
    
    public var selectedTextRange: UITextRange? {
        set {
            print("\(#function)")
        }
        
        get {
            print("\(#function)")
            return nil
        }
    }
    
    public var markedTextRange: UITextRange? {
        set {
            print("\(#function)")
        }
        
        get {
            print("\(#function)")
            return nil
        }
    }
    
    public var markedTextStyle: [NSAttributedString.Key : Any]? {
        set {
            print("\(#function)")
        }
        
        get {
            print("\(#function)")
            return nil
        }
    }
    
    public func setMarkedText(_ markedText: String?, selectedRange: NSRange) {
        print("\(#function)")
    }
    
    public func unmarkText() {
        print("\(#function)")
    }
    
    public var beginningOfDocument: UITextPosition {
        print("\(#function)")
        return UITextPosition.init()
    }
    
    public var endOfDocument: UITextPosition {
        print("\(#function)")
        return UITextPosition.init()
    }
    
    public func textRange(from fromPosition: UITextPosition, to toPosition: UITextPosition) -> UITextRange? {
        print("\(#function)")
        return nil
    }
    
    public func position(from position: UITextPosition, offset: Int) -> UITextPosition? {
        print("\(#function)")
        return nil
    }
    
    public func position(from position: UITextPosition, in direction: UITextLayoutDirection, offset: Int) -> UITextPosition? {
        print("\(#function)")
        return nil
    }
    
    public func compare(_ position: UITextPosition, to other: UITextPosition) -> ComparisonResult {
        print("\(#function)")
        return ComparisonResult.orderedAscending
    }
    
    public func offset(from: UITextPosition, to toPosition: UITextPosition) -> Int {
        print("\(#function)")
        return 0
    }
    
    public var inputDelegate: UITextInputDelegate? {
        set {
            print("\(#function)")
        }
        
        get {
            print("\(#function)")
            return nil
        }
    }
    
    public var tokenizer: UITextInputTokenizer  {
        print("\(#function)")
        return self
    }
    
    public func position(within range: UITextRange, farthestIn direction: UITextLayoutDirection) -> UITextPosition? {
        print("\(#function)")
        return nil
    }
    
    public func characterRange(byExtending position: UITextPosition, in direction: UITextLayoutDirection) -> UITextRange? {
        print("\(#function)")
        return nil
    }
    
    public func baseWritingDirection(for position: UITextPosition, in direction: UITextStorageDirection) -> NSWritingDirection {
        print("\(#function)")
        return NSWritingDirection.natural
    }
    
    public func setBaseWritingDirection(_ writingDirection: NSWritingDirection, for range: UITextRange) {
        print("\(#function)")
    }
    
    public func firstRect(for range: UITextRange) -> CGRect {
        print("\(#function)")
        return CGRect.zero
    }
    
    public func caretRect(for position: UITextPosition) -> CGRect {
        print("\(#function)")
        return CGRect.zero
    }
    
    public func selectionRects(for range: UITextRange) -> [UITextSelectionRect] {
        print("\(#function)")
        return []
    }
    
    public func closestPosition(to point: CGPoint) -> UITextPosition? {
        print("\(#function)")
        return nil
    }
    
    public func closestPosition(to point: CGPoint, within range: UITextRange) -> UITextPosition? {
        print("\(#function)")
        return nil
    }
    
    public func characterRange(at point: CGPoint) -> UITextRange? {
        print("\(#function)")
        return nil
    }
    
    public var hasText: Bool {
        print("\(#function)")
        return false
    }
    
    public func deleteBackward() {
        print("\(#function)")
    }
    
    public func rangeEnclosingPosition(_ position: UITextPosition, with granularity: UITextGranularity, inDirection direction: UITextDirection) -> UITextRange? {
        print("\(#function)")
        return nil
    }
    
    public func isPosition(_ position: UITextPosition, atBoundary granularity: UITextGranularity, inDirection direction: UITextDirection) -> Bool {
        print("\(#function)")
        return false
    }
    
    public func position(from position: UITextPosition, toBoundary granularity: UITextGranularity, inDirection direction: UITextDirection) -> UITextPosition? {
        print("\(#function)")
        return nil
    }
    
    public func isPosition(_ position: UITextPosition, withinTextUnit granularity: UITextGranularity, inDirection direction: UITextDirection) -> Bool {
        print("\(#function)")
        return false
    }
    
    public override var canBecomeFirstResponder: Bool {
        print("\(#function)")
        print("canBecomeFirstResponder")
        return true
    }
    
    required init(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }
    
}

#endif
