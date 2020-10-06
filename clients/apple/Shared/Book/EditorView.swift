import SwiftUI
import SwiftLockbookCore
import Combine

struct EditorView: View, Equatable {
    /// Define an always equality so that this view doesn't reload once it's initialized
    static func == (lhs: EditorView, rhs: EditorView) -> Bool {
        lhs.meta.id == rhs.meta.id
    }
    
    @ObservedObject var core: Core
    @ObservedObject var buffer: Buffer
    let meta: FileMetadata
        
    var body: some View {
        let baseEditor = TextEditor(text: $buffer.content)
            .padding(0.1)
            .navigationTitle(meta.name)
            .disabled(!buffer.succeeded)
            .onAppear {
                switch core.api.getFile(id: meta.id) {
                case .success(let decrypted):
                    buffer.content = decrypted.secret
                    buffer.succeeded = true
                case .failure(let err):
                    core.displayError(error: err)
                    buffer.succeeded = false
                }
            }
            .onDisappear {
                switch buffer.save() {
                case .success(_):
                    buffer.succeeded = true
                case .failure(let err):
                    core.displayError(error: err)
                    buffer.succeeded = false
                }
            }
        
        
        #if os(iOS)
        baseEditor
            .navigationBarItems(trailing: makeStatus())
        #else
        baseEditor
            .toolbar(content: {
                ToolbarItem(placement: .automatic) { makeStatus() }
            })
        #endif
    }
    
    func makeStatus() -> some View {
        switch buffer.status {
        case .Inactive:
            return Image(systemName: "slash.circle")
                .foregroundColor(.secondary)
                .opacity(0.4)
        case .Succeeded:
            return Image(systemName: "checkmark.circle")
                .foregroundColor(.green)
                .opacity(0.6)
        case .Failed:
            return Image(systemName: "xmark.circle")
                .foregroundColor(.red)
                .opacity(0.6)
        }
    }
    
    init(core: Core, meta: FileMetadata) {
        self.core = core
        self.meta = meta
        self.buffer = Buffer(meta: meta, initialContent: "<PLACEHOLDER>", core: core)
    }
}

struct EditorView_Previews: PreviewProvider {
    static var previews: some View {
        NavigationView {
            EditorView(core: Core(), meta: FakeApi().fileMetas[0])
        }
    }
}

class Buffer: ObservableObject {
    let meta: FileMetadata
    private var cancellables: Set<AnyCancellable> = []
    let core: Core
    @Published var content: String
    @Published var succeeded: Bool = false
    @Published var status: SaveStatus = .Inactive
    
    init(meta: FileMetadata, initialContent: String, core: Core) {
        self.meta = meta
        self.core = core
        self.content = initialContent
        
        $content
            .debounce(for: 0.2, scheduler: RunLoop.main)
            .sink { _ in
                self.status = .Inactive
            }
            .store(in: &cancellables)
        
        $content
            .debounce(for: 1, scheduler: DispatchQueue.global(qos: .background))
            .filter({ _ in self.succeeded })
            .flatMap { _ in
                Future<Void, ApplicationError> { promise in
                    promise(self.save())
                }
            }
            .eraseToAnyPublisher()
            .receive(on: RunLoop.main)
            .sink(receiveCompletion: { (err) in
                self.status = .Failed
            }, receiveValue: { (input) in
                self.status = .Succeeded
            })
            .store(in: &cancellables)
        
    }
    
    func save() -> Result<Void, ApplicationError> {
        switch core.api.updateFile(id: meta.id, content: content) {
        case .success(_):
            return .success(())
        case .failure(let err):
            return .failure(err)
        }
    }
}

enum SaveStatus {
    case Succeeded
    case Failed
    case Inactive
}
