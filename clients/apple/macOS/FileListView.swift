//
//  FileListView.swift
//  Lockbook (macOS)
//
//  Created by Parth Mehrotra on 1/11/21.
//

import SwiftUI
import SwiftLockbookCore

struct FileListView: View {
    
    @State var selectedItem: FileMetadata? = nil
    
    @ObservedObject var core: Core
    let currentFolder: FileMetadata
    let account: Account
    
    init(core: Core, currentFolder: FileMetadata, account: Account) {
        self.core = core
        self.account = account
        self.currentFolder = currentFolder
    }
    
    var body: some View {
        OutlineSection(core: core, root: currentFolder, selectedItem: $selectedItem)
        if selectedItem != nil {
            EditorLoader(core: core, meta: selectedItem!)
        }
    }
}
