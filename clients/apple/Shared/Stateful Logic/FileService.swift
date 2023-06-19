import Foundation
import SwiftUI
import SwiftLockbookCore

class FileService: ObservableObject {
    let core: LockbookApi

    @Published var root: File? = nil
    @Published var idsAndFiles: [UUID:File] = [:]
    @Published var suggestedDocs: [File]? = nil
    var files: [File] {
        get {
            Array(idsAndFiles.values)
        }
    }
    var successfulAction: FileAction? = nil

    // File Service keeps track of the parent being displayed on iOS. Since this functionality is not used for macOS, it is conditionally compiled.
#if os(iOS)
    @Published var path: [File] = []

    var parent: File? {
        get {
            path.last
        }
    }

    func childrenOfParent() -> [File] {
        return childrenOf(path.last)
    }

    func upADirectory() {
        self.path.removeLast()
    }

    func intoChildDirectory(_ file: File) {
                self.path.append(file)
    }

    func pathBreadcrumbClicked(_ file: File) {
        DispatchQueue.main.async {
            withAnimation {
                if let firstIndex = self.path.firstIndex(of: file) {
                    self.path.removeSubrange(firstIndex + 1...self.path.count - 1)
                }
            }
        }
    }
#endif

    func childrenOf(_ meta: File?) -> [File] {
        var file: File
        if meta == nil {
            guard let theRoot = root else {
                return []
            }
            file = theRoot
        } else {
            file = meta!
        }

        var toBeSorted = files.filter {
            $0.parent == file.id && $0.parent != $0.id
        }

        toBeSorted.sort()

        return toBeSorted
    }

    func childrenOfRoot() -> [File] {
        let root = root!
        return childrenOf(root)
    }

    init(_ core: LockbookApi) {
        self.core = core

        if DI.accounts.account != nil {
            refresh()
        }
    }

    // TODO in the future we should pop one of these bad boys up during this operation
    // https://github.com/elai950/AlertToast
    func moveFile(id: UUID, newParent: UUID) {
        print("moving file")
        DispatchQueue.global(qos: .userInteractive).async {
            let operation = self.core.moveFile(id: id, newParent: newParent)

            DispatchQueue.main.async {
                switch operation {
                case .success(_):
                    self.successfulAction = .move
                    self.refresh()
                    DI.status.checkForLocalWork()
                case .failure(let error):
                    switch error.kind {
                    case .UiError(let uiError):
                        switch uiError {
                        case .FolderMovedIntoItself:
                            DI.errors.errorWithTitle("Move Error", "Cannot move a folder into itself or one of it's children")
                        case .TargetParentHasChildNamedThat:
                            DI.errors.errorWithTitle("Move Error", "Target folder has a child named that")
                        default:
                            DI.errors.handleError(error)
                        }
                    default:
                        DI.errors.handleError(error)
                    }
                }
            }
        }
    }

    func moveFileSync(id: UUID, newParent: UUID) -> Bool {
        print("moving file")
        let operation = core.moveFile(id: id, newParent: newParent)

        switch operation {
        case .success(_):
            self.successfulAction = .move
            refresh()
            DI.status.checkForLocalWork()
            return true
        case .failure(let error):
            switch error.kind {
            case .UiError(let uiError):
                switch uiError {
                case .FolderMovedIntoItself:
                    DI.errors.errorWithTitle("Move Error", "Cannot move a folder into itself or one of it's children")
                case .TargetParentHasChildNamedThat:
                    DI.errors.errorWithTitle("Move Error", "Target folder has a child named that")
                default:
                    DI.errors.handleError(error)
                }
            default:
                DI.errors.handleError(error)
            }
            return false
        }
    }
    
    func importFilesSync(sources: [String], destination: UUID) -> Bool {
        print("importing files")
        let operation = core.importFiles(sources: sources, destination: destination)

        switch operation {
        case .success(_):
            self.successfulAction = .importFiles
            refresh()
            DI.status.checkForLocalWork()
            return true
        case .failure(let error):
            DI.errors.handleError(error)
            return false
        }
    }


    func deleteFile(id: UUID) {
        DispatchQueue.global(qos: .userInitiated).async {
            let operation = self.core.deleteFile(id: id)

            DispatchQueue.main.async {
                switch operation {
                case .success(_):
                    if DI.documentLoader.meta?.id == id {
                        DI.documentLoader.deleted = true
                    }
                    self.successfulAction = .delete
                    self.refresh()
                    DI.status.checkForLocalWork()
                case .failure(let error):
                    DI.errors.handleError(error)
                }
            }
        }
    }

    func renameFile(id: UUID, name: String) {
        DispatchQueue.global(qos: .userInteractive).async {
            let operation = self.core.renameFile(id: id, name: name)

            DispatchQueue.main.async {
                switch operation {
                case .success(_):
                    self.successfulAction = .rename
                    self.refresh()
                    DI.status.checkForLocalWork()
                case .failure(let error):
                    switch error.kind {
                    case .UiError(let uiError):
                        switch uiError {
                        case .FileNameNotAvailable:
                            DI.errors.errorWithTitle("Rename Error", "File with that name exists already")
                        case .NewNameContainsSlash:
                            DI.errors.errorWithTitle("Rename Error", "Filename cannot contain slash")
                        case .NewNameEmpty:
                            DI.errors.errorWithTitle("Rename Error", "Filename cannot be empty")
                        default:
                            DI.errors.handleError(error)
                        }
                    default:
                        DI.errors.handleError(error)
                    }
                }
            }
        }
    }

    func filesToExpand(pathToRoot: [File], currentFile: File) -> [File] {
        if(currentFile.isRoot) {
            return []
        }

        let parentFile = idsAndFiles[currentFile.parent]!

        var pathToRoot = filesToExpand(pathToRoot: pathToRoot, currentFile: parentFile)

        if(currentFile.fileType == .Folder) {
            pathToRoot.append(currentFile)
        }

        return pathToRoot
    }
    
    func refreshSuggestedDocs() {
        DispatchQueue.global(qos: .userInitiated).async {
            switch self.core.suggestedDocs() {
            case .success(let ids):
                var suggestedDocs: [File] = []
                    
                for id in ids.filter({ self.idsAndFiles[$0] != nil }) {
                    switch self.core.getFileById(id: id) {
                    case .success(let meta):
                        suggestedDocs.append(meta)
                    case .failure(let error):
                        if error.kind != .UiError(.NoFileWithThatId) {
                            DI.errors.handleError(error)
                        }
                    }
                }
                    
                DispatchQueue.main.async {
                    self.suggestedDocs = suggestedDocs
                }
            case .failure(let error):
                DI.errors.handleError(error)
            }
        }
    }

    func refresh() {
        DispatchQueue.global(qos: .userInteractive).async {
            let allFiles = self.core.listFiles()

            DispatchQueue.main.async {
                switch allFiles {
                case .success(let files):
                    self.idsAndFiles = Dictionary(uniqueKeysWithValues: files.map { ($0.id, $0) })
                    self.refreshSuggestedDocs()
                    self.files.forEach {
                        self.notifyDocumentChanged($0)
                        if self.root == nil && $0.id == $0.parent {
                            self.root = $0

                            #if os(iOS)
                            if(self.path.isEmpty) {
                                self.path.append($0)
                            }
                            #endif
                        }
                    }
                    self.openFileChecks()
                case .failure(let error):
                    DI.errors.handleError(error)
                }
            }
        }
    }

    private func openFileChecks() {
        if let openedMeta = DI.currentDoc.selectedDocument {
            let maybeMeta = idsAndFiles[openedMeta.id]
            
            if maybeMeta == nil {
                DI.documentLoader.deleted = true
            } else if openedMeta != maybeMeta {
                DI.currentDoc.selectedDocument = maybeMeta
            }
        }
    }

    private func notifyDocumentChanged(_ meta: File) {
        if let openDocument = DI.documentLoader.meta, meta.id == openDocument.id, meta.lastModified != openDocument.lastModified {
            DI.documentLoader.updatesFromCoreAvailable(meta)
        }
    }
}

public enum FileAction {
    case move
    case rename
    case delete
    case createFolder
    case importFiles
}
