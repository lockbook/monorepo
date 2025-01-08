import SwiftUI
import SwiftWorkspace

struct FileTreeView: View {
    
    @EnvironmentObject var workspaceState: WorkspaceState
    @EnvironmentObject var fileTreeModel: FileTreeViewModel
    
    var root: File

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 2) {
                FileRowView(file: root, level: -1)
                
                Spacer()
            }
            .listStyle(.sidebar)
            .frame(minWidth: 10, maxWidth: .infinity, maxHeight: .infinity)
        }.contextMenu {
            FileRowContextMenu(file: root)
        }
        .refreshable {
            workspaceState.requestSync()
        }
        .sheet(item: $fileTreeModel.sheetInfo) { info in
            switch info {
            case .create(parent: let parent):
                EmptyView()
            case .rename(file: let file):
                EmptyView()
            case .select(action: let action):
                EmptyView()
            case .share(file: let file):
                EmptyView()
            }
        }
        .padding(.leading)
    }
}

#Preview {
    FileTreeView(root: (MainState.lb as! MockLb).file0)
        .environmentObject(FileTreeViewModel(workspaceState: WorkspaceState()))
        .environmentObject(FilesViewModel(workspaceState: WorkspaceState()))
        .environmentObject(WorkspaceState())
}

struct FileRowView: View {
    let file: File
    let level: CGFloat
    
    var children: [File] {
        get {
            filesModel.childrens[file.id] ?? []
        }
    }

    var isLeaf: Bool {
        get {
            children.isEmpty
        }
    }
        
    var isOpen: Bool {
        get {
            fileTreeModel.openFiles.contains(file.id) || fileTreeModel.implicitlyOpenFiles.contains(file.id)
        }
    }
        
    @EnvironmentObject var filesModel: FilesViewModel
    @EnvironmentObject var fileTreeModel: FileTreeViewModel
    @EnvironmentObject var workspaceState: WorkspaceState
        
    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            if level != -1 {
                fileRow
                    .onTapGesture {
                        openFile()
                    }
            }
            
            if !isLeaf && (isOpen || level == -1) {
                ForEach(children) { child in
                    FileRowView(file: child, level: level + 1)
                }
            }
        }
        .contextMenu {
            FileRowContextMenu(file: file)
        }
    }
    
    var fileRow: some View {
        HStack {
            Image(systemName: FileIconHelper.fileToSystemImageName(file: file))
                .font(.system(size: 16))
                .frame(width: 16)
                .foregroundColor(file.isFolder ? .accentColor : (false ? .white : .secondary ))
                        
            Text(file.name)
                .lineLimit(1)
                .truncationMode(.tail)
                .allowsTightening(true)
            
            Spacer()
            
            if !isLeaf {
                Image(systemName: "chevron.forward")
                    .renderingMode(.template)
                    .resizable()
                    .scaledToFit()
                    .frame(width: 10, height: 10)
                    .rotationEffect(Angle.degrees(isOpen ? 90 : 0))
                    .foregroundColor(.accentColor)
            }

        }
        .padding(.vertical, 9)
        .contentShape(Rectangle())
        .padding(.leading, level * 20 + 5)
        .padding(.trailing, 10)
    }
    
    func openFile() {
        if file.isFolder {
            workspaceState.selectedFolder = file.id
            
            withAnimation {
                let _ = fileTreeModel.openFiles.insert(file.id)
            }
        } else {
            workspaceState.requestOpenDoc(file.id)
        }
    }
}

struct FileRowContextMenu: View {
    let file: File
    
    @EnvironmentObject var fileTreeModel: FileTreeViewModel
    @EnvironmentObject var filesModel: FilesViewModel
    
    var body: some View {
        VStack {
            if file.isFolder {
                Button(action: {
                    filesModel.createDoc(parent: file.id, isDrawing: false)
                }) {
                    Label("Create a document", systemImage: "doc.fill")
                }
                Button(action: {
                    filesModel.createDoc(parent: file.id, isDrawing: true)
                }) {
                    Label("Create a drawing", systemImage: "pencil.tip.crop.circle.badge.plus")
                }
                Button(action: {
                    fileTreeModel.sheetInfo = .create(parent: file)
                }) {
                    Label("Create a folder", systemImage: "folder.fill")
                }
            }
            
            if !file.isRoot {
                Button(action: {
                    fileTreeModel.sheetInfo = .rename(file: file)
                }) {
                    Label("Rename", systemImage: "pencil.circle.fill")
                }

                Button(action: {
                    fileTreeModel.sheetInfo = .select(action: .Move(ids: [file.id]))
                }) {
                    Label("Move", systemImage: "arrow.up.and.down.and.arrow.left.and.right")
                }
                
                Divider()
                
                Button(action: {
                    fileTreeModel.sheetInfo = .share(file: file)
                }) {
                    Label("Share", systemImage: "person.wave.2.fill")
                }

                Button(action: {
//                    exportFilesAndShowShareSheet(metas: [meta])
                }) {
                    Label("Share externally to...", systemImage: "square.and.arrow.up.fill")
                }
                
                if file.type == .document {
                    Button(action: {
                        ClipboardHelper.copyFileLink(file.id)
                    }) {
                        Label("Copy file link", systemImage: "link")
                    }
                }
                
                Divider()
                
                Button(role: .destructive, action: {
//                    DI.sheets.deleteConfirmationInfo = [meta]
                }) {
                    Label("Delete", systemImage: "trash.fill")
                }
            }
        }
    }
}
