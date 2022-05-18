import SwiftUI
import SwiftLockbookCore

struct OutlineSection: View {
    
    @EnvironmentObject var onboarding: OnboardingService
    @EnvironmentObject var files: FileService
    @EnvironmentObject var state: SheetState
    
    var root: DecryptedFileMetadata

    var children: [DecryptedFileMetadata] {
        files.files.filter {
            $0.parent == root.id && $0.id != root.id
        }
    }
    
    var isLeaf: Bool {
        children.isEmpty
    }
    
    var body: some View {
        let rootOutlineBranch = OutlineBranch(file: root, level: -1)
        ScrollView {
            VStack(alignment: .leading, spacing: 2) {
                // The padding in the section header is there to adjust for the inset hack.
                rootOutlineBranch
                Spacer()
            }
            .listStyle(SidebarListStyle())
            .frame(minWidth: 10, maxWidth: .infinity, maxHeight: .infinity)
            .padding()
            // A hack for list row insets not working. This hack also applies to the section header though.
        }.contextMenu {
            OutlineContextMenu (meta: root, branchState: nil)
        }
        .navigationTitle(files.root!.decryptedName)
    }
}
