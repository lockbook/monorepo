import SwiftUI
import SwiftLockbookCore
import PencilKit
import SwiftEditor

struct DocumentView: View {
    
    let meta: File
    
    @EnvironmentObject var model: DocumentLoader
#if os(iOS)
    @EnvironmentObject var toolbar: ToolbarModel
    @EnvironmentObject var current: CurrentDocument
#endif
    
    var body: some View {
        Group {
            if meta != model.meta || model.loading {
                ProgressView()
                    .onAppear {
                        model.startLoading(meta)
                    }
                    .title(meta.name)
            } else if model.error != "" {
                Text("errors while loading: \(model.error)")
            } else if model.deleted {
                Text("\(meta.name) was deleted.")
            } else {
                if let type = model.type {
                    switch type {
                    case .Image:
                        if let img = model.image {
                            ScrollView([.horizontal, .vertical]) {
                                img
                            }.title(meta.name)
                        }
#if os(iOS)
                    case .Drawing:
                        DrawingView(
                            model: model,
                            toolPicker: toolbar
                        )
                        .navigationBarTitle(meta.name, displayMode: .inline)
                        .toolbar {
                            ToolbarItemGroup(placement: .bottomBar) {
                                Spacer()
                                DrawingToolbar(toolPicker: toolbar)
                                Spacer()
                            }
                        }
#endif

                    case .Markdown:
                        if let editorState = model.textDocument {
                            MarkdownEditor(documentName: meta.name, editorState: editorState, editor: EditorView(editorState))
                        }
                    case .Unknown:
                        Text("\(meta.name) cannot be opened on this device.")
                            .title(meta.name)
                    }
                }
            }
        }
        .onDisappear {
            DI.files.refreshSuggestedDocs()
        }
    }
}

extension View {
    func title(_ name: String) -> some View {
#if os(macOS)
        return self.navigationTitle(name)
#else
        return self.navigationBarTitle(name, displayMode: .inline)
#endif
    }
}

struct MarkdownEditor: View {
    var documentName: String
    
    @ObservedObject var editorState: EditorState
    let editor: EditorView
    
    @Environment(\.colorScheme) var colorScheme
            
    var body: some View {
        #if os(iOS)
        VStack {
            editor
                
            ScrollView(.horizontal) {
                toolbar
                    .padding(.bottom, 8)
                    .padding(.horizontal)
            }
        }
        .title(documentName)
        #else
        VStack {
            toolbar
                .padding(.top, 9)
                .padding(.horizontal)
//                .padding(.bottom, 3)
            
            editor
        }
        .title(documentName)
        #endif
        
    }
    
    var toolbar: some View {
        HStack(spacing: 20) {
            HStack(spacing: 15) {
                Menu(content: {
                    Button("Heading 1") {
                        editor.header(headingSize: 1)
                    }
                    
                    Button("Heading 2") {
                        editor.header(headingSize: 2)
                    }
                    
                    Button("Heading 3") {
                        editor.header(headingSize: 3)
                    }
                    
                    Button("Heading 4") {
                        editor.header(headingSize: 4)
                    }
                }, label: {
                    Image(systemName: "h.square")
                        .foregroundColor(.primary)
                        .padding(2)
                })
                .menuStyle(.borderlessButton)
                .frame(width: 30)
                .padding(3)
                .background(editorState.isHeadingSelected ? .gray.opacity(0.2) : .clear)
                .cornerRadius(5)
            }
            
            Divider()
                .frame(height: 20)
            
            HStack(spacing: 15) {
                Button(action: {
                    editor.bold()
                }) {
                    MarkdownEditorImage(systemImageName: "bold", isSelected: false)
                }
                .buttonStyle(.borderless)
                
                Button(action: {
                    editor.italic()
                }) {
                    MarkdownEditorImage(systemImageName: "italic", isSelected: false)
                }
                .buttonStyle(.borderless)
                
                Button(action: {
                    editor.code()
                }) {
                    MarkdownEditorImage(systemImageName: "greaterthan.square", isSelected: false)
                }
                .buttonStyle(.borderless)
                
            }
            
            Divider()
                .frame(height: 20)
                        
            HStack(spacing: 15) {
                Button(action: {
                    editor.bulletedList()
                }) {
                    MarkdownEditorImage(systemImageName: "list.bullet", isSelected: editorState.isBulletListSelected)
                }
                .buttonStyle(.borderless)
                
                Button(action: {
                    editor.numberedList()
                }) {
                    MarkdownEditorImage(systemImageName: "list.number", isSelected: editorState.isNumberListSelected)
                }
                .buttonStyle(.borderless)
                
                Button(action: {
                    editor.checkedList()
                }) {
                    MarkdownEditorImage(systemImageName: "checklist", isSelected: editorState.isChecklistSelected)
                }
                .buttonStyle(.borderless)
            }
            
            #if os(iOS)
            
            Divider()
                .frame(height: 20)
            
            HStack(spacing: 15) {
                
                Button(action: {
                    editor.tab(deindent: false)
                }) {
                    MarkdownEditorImage(systemImageName: "arrow.right.to.line.compact", isSelected: false)
                }
                .buttonStyle(.borderless)
                
                Button(action: {
                    editor.tab(deindent: true)
                }) {
                    MarkdownEditorImage(systemImageName: "arrow.left.to.line.compact", isSelected: false)
                }
                .buttonStyle(.borderless)
            }
            
            #endif
            
            Spacer()
        }
    }
}

struct MarkdownEditorImage: View {
    let systemImageName: String
    var isSelected: Bool
    
    var body: some View {
        Image(systemName: systemImageName)
            .padding(5)
            .foregroundColor(.primary)
//            .foregroundColor(isSelected ? .blue : .primary)
            .background(isSelected ? .gray.opacity(0.2) : .clear)
            .cornerRadius(5)
    }
}
