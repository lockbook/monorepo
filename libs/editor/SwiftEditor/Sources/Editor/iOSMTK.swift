#if os(iOS)
import UIKit
import MetalKit
import Bridge

public class iOSMTK: MTKView, MTKViewDelegate, UITextInput, UITextInputTokenizer {
    var editorHandle: UnsafeMutableRawPointer?
    
    override init(frame frameRect: CGRect, device: MTLDevice?) {
        super.init(frame: frameRect, device: device)
        
        let metalLayer = UnsafeMutableRawPointer(Unmanaged.passRetained(self.layer).toOpaque())
        self.editorHandle = init_editor(metalLayer, test, false) // todo
        
        self.isPaused = true
        self.enableSetNeedsDisplay = true
        self.delegate = self
        self.addGestureRecognizer(UIPanGestureRecognizer(target: self, action: #selector(didPan(_:))))
    }
    
    @objc private func didPan(_ sender: UIPanGestureRecognizer) {
        print(sender.translation(in: self).y)
        print(self.frame)
        scroll_wheel(editorHandle, Float(sender.translation(in: self).y) / 3)
        self.setNeedsDisplay(self.frame)
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
        insert_text(editorHandle, text)
        self.setNeedsDisplay(self.frame)
    }
    
    public func text(in range: UITextRange) -> String? {
        if range == nil { return nil } // todo: remove this
        let range = range as! LBTextRange
        let result = text_in_range(editorHandle, range.c)
        let str = String(cString: result!)
        free_text(UnsafeMutablePointer(mutating: result))
        return str
    }
    
    
    public func replace(_ range: UITextRange, withText text: String) {
        let range = range as! LBTextRange
        replace_text(editorHandle, range.c, text)
        self.setNeedsDisplay(self.frame)
    }
    
    public var selectedTextRange: UITextRange? {
        set {
            print("set \(#function)")
        }
        
        get {
            print("get \(#function)")
            return nil
        }
    }
    
    public var markedTextRange: UITextRange? {
        set {
            print("set \(#function)")
        }
        
        get {
            print("get \(#function)")
            return nil
        }
    }
    
    public var markedTextStyle: [NSAttributedString.Key : Any]? {
        set {
            print("set \(#function)")
        }
        
        get {
            print("get \(#function)")
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
        LBTextPos(c: beginning_of_document(editorHandle))
    }
    
    public var endOfDocument: UITextPosition {
        LBTextPos(c: end_of_document(editorHandle))
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
            print("set \(#function)")
        }
        
        get {
            print("get \(#function)")
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
        has_text(editorHandle)
    }
    
    public func deleteBackward() {
        backspace(editorHandle)
        self.setNeedsDisplay(self.frame)
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
    
    public override func touchesBegan(_ touches: Set<UITouch>, with event: UIEvent?) {
        let point = Unmanaged.passUnretained(touches.first!).toOpaque()
        let value = UInt64(UInt(bitPattern: point))
        let location = touches.first!.location(in: self)
        touches_began(editorHandle, value, Float(location.x), Float(location.y), Float(touches.first?.force ?? 0))
        self.setNeedsDisplay(self.frame)
    }
    
    public override func touchesMoved(_ touches: Set<UITouch>, with event: UIEvent?) {
        let point = Unmanaged.passUnretained(touches.first!).toOpaque()
        let value = UInt64(UInt(bitPattern: point))
        let location = touches.first!.location(in: self)
        touches_moved(editorHandle, value, Float(location.x), Float(location.y), Float(touches.first?.force ?? 0))
        self.setNeedsDisplay(self.frame)
    }
    
    public override func touchesEnded(_ touches: Set<UITouch>, with event: UIEvent?) {
        let point = Unmanaged.passUnretained(touches.first!).toOpaque()
        let value = UInt64(UInt(bitPattern: point))
        let location = touches.first!.location(in: self)
        touches_ended(editorHandle, value, Float(location.x), Float(location.y), Float(touches.first?.force ?? 0))
        self.setNeedsDisplay(self.frame)
    }
    
    public override func touchesCancelled(_ touches: Set<UITouch>, with event: UIEvent?) {
        let point = Unmanaged.passUnretained(touches.first!).toOpaque()
        let value = UInt64(UInt(bitPattern: point))
        let location = touches.first!.location(in: self)
        touches_cancelled(editorHandle, value, Float(location.x), Float(location.y), Float(touches.first?.force ?? 0))
        self.setNeedsDisplay(self.frame)
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

class LBTextRange: UITextRange {
    let c: CTextRange
    
    init(c: CTextRange) {
        self.c = c
    }
}

class LBTextPos: UITextPosition {
    let c: CTextPosition
    
    init(c: CTextPosition) {
        self.c = c
    }
}
#endif

let test = """
# Lockbook

![Discord](https://img.shields.io/discord/1014184997751619664?label=Discord&style=plastic)

## About
_The private, polished note-taking platform._

Privacy shouldn't be a compromise. That's why we made Lockbook, a companion for recording thoughts on all your devices. Record, sync, and share your notes with apps engineered to feel like home on every platform. We collect no personal information and encrypt your notes so even _we_ can’t see them. Don’t take our word for it: Lockbook is 100% open-source.

### Polished
We built Lockbook for everyday use because we use Lockbook everyday. We need a note-taking app that doesn't make trade-offs with respect to speed, stability, efficiency, device integration, or delightfulness. The only way to have that is to put in the effort, including writing native apps for every platform, and we can't wait for you to try them.

### Secure
Keep your thoughts to yourself. Your notes are encrypted with keys that are generated on your devices and stay on your devices. The only people that can see your notes are you and the users you share them with. No one else, including infrastructure providers, state actors, and Lockbook employees, can see your notes.

### Private
Know your customer? We sure don't. We don't collect your email, phone number, or name. We don't even need a password. Lockbook is for people with better things to worry about than privacy.

### Honest
Be the customer, not the product. We make money by selling a note-taking app, not your data.

| Payment Option | Monthly Fee    |
|----------------|----------------|
| Monthly        | $2.99 per 30gb |

### Developer Friendly
We also provide a CLI tool that will fit right into your favorite chain of piped-together unix commands. Search your notes with `fzf`, edit them with `vim`, and schedule backups with `cron`. When scripting doesn't cut it, use our Rust library for a robust programmatic interface with Lockbook.

## Feature Matrix

<details>
<summary>Legend</summary>

+ ✅ Done
+ 🏗 In Progress
+ 📆 Planned
+ ⛔️ Not Planned

</details>

### Account Management

|                    |  [CLI]  |  [Linux]  |  [Android]  |  [Windows]  |  [iOS/iPadOS]  |  [macOS]  |
|--------------------|:-------:|:---------:|:-----------:|:-----------:|:--------------:|:---------:|
| New Account        |   ✅     |    ✅     |     ✅      |     ✅       |      ✅        |    ✅     |
| QR Import          |   ⛔️     |    📆     |     ✅      |     📆       |      ✅        |    📆     |
| Import Account     |   ✅     |    ✅     |     ✅      |     ✅       |      ✅        |    ✅     |
| Space Utilized     |   ✅     |    ✅     |     ✅      |     ✅       |      ✅        |    ✅     |
| Billing            |   ✅     |    ✅     |     ✅      |     📆       |      📆        |    📆     |

### File Operations

|                       |  [CLI]  |  [Linux]  |  [Android]  |  [Windows]  |  [iOS/iPadOS]  |  [macOS]  |
|-----------------------|:-------:|:---------:|:-----------:|:-----------:|:--------------:|:---------:|
| Rename                |   ✅     |    ✅     |     ✅      |     ✅       |      ✅        |    ✅     |
| Move                  |   ✅     |    ✅     |     ✅      |     ✅       |      ✅        |    ✅     |
| Delete                |   ✅     |    ✅     |     ✅      |     ✅       |      ✅        |    ✅     |
| Sync                  |   ✅     |    ✅     |     ✅      |     ✅       |      ✅        |    ✅     |
| Export file to host   |   ✅     |    ✅     |     ✅      |     📆       |      📆        |    📆     |
| Import file from host |   ✅     |    ✅     |     📆      |     📆       |      📆        |    📆     |
| Sharing               |   ✅     |    📆     |     📆      |     📆       |      📆        |    📆     |

### Document Types

|                       |  [CLI]  |  [Linux]  |  [Android]  |  [Windows]  |  [iOS/iPadOS]  |  [macOS]  |
|-----------------------|:-------:|:---------:|:-----------:|:-----------:|:--------------:|:---------:|
| Text                  |   ✅     |    ✅     |     ✅      |     ✅       |      ✅        |    ✅     |
| Markdown              |   ✅     |    ✅     |     ✅      |     📆       |      ✅        |    ✅     |
| Drawings              |   ✅     |    🏗     |     ✅      |     🏗       |      ✅        |    ✅     |
| Images                |   ✅     |    ✅     |     ✅      |     📆       |      📆        |    📆     |
| PDFs                  |   📆     |    📆     |     ✅      |     📆       |      📆        |    📆     |
| Todo lists            |   📆     |    📆     |     📆      |     📆       |      📆        |    📆     |
| Document Linking      |   📆     |    ✅     |     📆      |     📆       |      📆        |    📆     |

# Further Reading

+ [System Architecture](design-tech/system-architecture.md)
+ [Data Model and Procedures](design-tech/data_model.md)

[Cli]: guides/install/cli.md
[Linux]: guides/install/linux.md
[Android]: guides/install/android.md
[Windows]: guides/install/windows.md
[macOS]: guides/install/macos.md
[iOS/iPadOS]: guides/install/iOS-iPadOS.md
"""
