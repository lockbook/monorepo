import SwiftUI
import SwiftWorkspace
import SwiftLockbookCore
import Foundation

struct ConstrainedHomeView: View {
    @EnvironmentObject var files: FileService
    @EnvironmentObject var search: SearchService
    
    @Binding var searchInput: String
    
    @Environment(\.isSearching) var isSearching
    @Environment(\.colorScheme) var colorScheme
    @Environment(\.dismissSearch) private var dismissSearch
    
    var body: some View {
        VStack {
            if search.isPathAndContentSearching {
                if search.isPathAndContentSearchInProgress {
                    ProgressView()
                        .frame(width: 20, height: 20)
                        .padding(.top)
                }
                
                if !search.pathAndContentSearchResults.isEmpty {
                    List(search.pathAndContentSearchResults) { result in
                        switch result {
                        case .PathMatch(_, let meta, let name, let path, let matchedIndices, _):
                            Button(action: {
                                DI.workspace.requestOpenDoc(meta.id)
                                dismissSearch()
                            }) {
                                SearchFilePathCell(name: name, path: path, matchedIndices: matchedIndices)
                            }
                        case .ContentMatch(_, let meta, let name, let path, let paragraph, let matchedIndices, _):
                            Button(action: {
                                DI.workspace.requestOpenDoc(meta.id)
                                dismissSearch()
                            }) {
                                SearchFileContentCell(name: name, path: path, paragraph: paragraph, matchedIndices: matchedIndices)
                            }
                        }
                    }
                    .listStyle(.inset)
                } else if !search.isPathAndContentSearchInProgress && !search.pathAndContentSearchQuery.isEmpty {
                    Text("No results.")
                        .font(.headline)
                        .foregroundColor(.gray)
                        .fontWeight(.bold)
                        .padding()
                    
                    Spacer()
                }
            } else {
                main
            }
        }
        .onChange(of: searchInput) { newInput in
            DI.search.search(query: newInput, isPathAndContentSearch: true)
        }
        .onChange(of: isSearching, perform: { newInput in
            if newInput {
                DI.search.startSearchThread(isPathAndContentSearch: true)
            } else {
                DI.search.endSearch(isPathAndContentSearch: true)
            }
        })
        .navigationBarTitle(files.parent.map{$0.name} ?? "")
    }
    
    var main: some View {
//        Group {
            List {
                if files.parent?.isRoot == true && files.suggestedDocs?.isEmpty != true {
                    Section(header: Text("Suggested")
                        .bold()
                        .foregroundColor(.primary)
                        .textCase(.none)
                        .font(.headline)
                        .padding(.bottom, 3)) {
                            SuggestedDocs(isiOS: true)
                        }
                }
                
                Section(header: Text("Files")
                    .bold()
                    .foregroundColor(.primary)
                    .textCase(.none)
                    .font(.headline)
                    .padding(.bottom, 3)) {
                        fileList
                    }
            }
//        }
    }
    
    var fileList: some View {
        ForEach(files.childrenOfParent()) { meta in
            FileCell(meta: meta)
                .padding(.horizontal)
                .padding(.vertical, 5)
                .background(colorScheme == .light ? .white : Color(uiColor: .secondarySystemBackground))
        }
        .listRowBackground(Color.clear)
        .listRowInsets(EdgeInsets())
        .listRowSeparator(.hidden)
    }
}

struct FileListView: View {
    @EnvironmentObject var sheets: SheetState
    @EnvironmentObject var fileService: FileService
    @EnvironmentObject var search: SearchService
    @EnvironmentObject var sync: SyncService
    @EnvironmentObject var workspace: WorkspaceState
    
    @Environment(\.colorScheme) var colorScheme

    @State var searchInput: String = ""
    @State var navigateToManageSub: Bool = false
    @State private var mainViewOffset = CGSize.zero
    @State private var mainViewOpacity: Double = 1
    
    @State private var hideOutOfSpaceAlert = UserDefaults.standard.bool(forKey: "hideOutOfSpaceAlert")
    
    var body: some View {
        ZStack {
            SearchWrapperView(
                searchInput: $searchInput,
                mainView: mainView,
                isiOS: true)
            .searchable(text: $searchInput, prompt: "Search")
        }
        .gesture(
            DragGesture().onEnded({ (value) in
                if value.translation.width > 50 && fileService.parent?.isRoot == false {
                    fileService.upADirectory()
                }
            }))
        .alert(isPresented: Binding(get: { sync.outOfSpace && !hideOutOfSpaceAlert }, set: {_ in sync.outOfSpace = false })) {
            Alert(
                title: Text("Out of Space"),
                message: Text("You have run out of space!"),
                primaryButton: .default(Text("Upgrade now"), action: {
                    navigateToManageSub = true
                }),
                secondaryButton: .default(Text("Don't show me this again"), action: {
                    hideOutOfSpaceAlert = true
                    UserDefaults.standard.set(hideOutOfSpaceAlert, forKey: "hideOutOfSpaceAlert")
                })
            )
        }
        .background(
            NavigationLink(destination: ManageSubscription(), isActive: $navigateToManageSub, label: {
                EmptyView()
            })
            .hidden()
        )
    }
    
    @ViewBuilder
    var mainView: some View {
        Group {
            List {
                if fileService.parent?.isRoot == true && fileService.suggestedDocs?.isEmpty != true {
                    Section(header: Text("Suggested")
                        .bold()
                        .foregroundColor(.primary)
                        .textCase(.none)
                        .font(.headline)
                        .padding(.bottom, 3)) {
                            SuggestedDocs(isiOS: true)
                        }
                        .offset(mainViewOffset)
                        .opacity(mainViewOpacity)
                }
                
                Section(header: Text("Files")
                    .bold()
                    .foregroundColor(.primary)
                    .textCase(.none)
                    .font(.headline)
                    .padding(.bottom, 3)) {
                        EmptyView()
                    }
                    .offset(mainViewOffset)
                    .opacity(mainViewOpacity)
            }
            .navigationBarTitle(fileService.parent.map{($0.name)} ?? "")
            .modifier(DragGestureViewModifier(onUpdate: { gesture in
                if fileService.parent?.isRoot == false && gesture.translation.width < 200 && gesture.translation.width > 0 {
                    mainViewOffset.width = gesture.translation.width
                }
            }, onEnd: { gesture in
                if gesture.translation.width > 100 && fileService.parent?.isRoot == false {
                    animateToParentFolder() {
                        fileService.upADirectory()
                    }
                } else {
                    withAnimation {
                        mainViewOffset.width = 0
                    }
                }
            }))
            
            FilePathBreadcrumb() { file in
                animateToParentFolder() {
                    fileService.pathBreadcrumbClicked(file)
                }
            }
            
            BottomBar(isiOS: true)
        }
    }

    var files: some View {
        let children = fileService.childrenOfParent()

        return ForEach(children) { meta in
            EmptyView()
        }
        .listRowBackground(Color.clear)
        .listRowInsets(EdgeInsets())
        .listRowSeparator(.hidden)
    }

    func animateToParentFolder(realParentUpdate: @escaping () -> Void) {
        withAnimation(.linear(duration: 0.2)) {
            mainViewOffset.width = 200
            mainViewOpacity = 0
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) {
            mainViewOffset.width = -200
            mainViewOpacity = 1

            realParentUpdate()

            DispatchQueue.main.asyncAfter(deadline: .now() + 0.1) {
                withAnimation(.linear(duration: 0.1)) {
                    mainViewOffset.width = 0
                }
            }
        }
    }
}

struct DragGestureViewModifier: ViewModifier {
    @GestureState private var isDragging: Bool = false
    @State private var gestureState: GestureStatus = .idle

    var onUpdate: (DragGesture.Value) -> Void
    var onEnd: (DragGesture.Value) -> Void

    func body(content: Content) -> some View {
        content
            .gesture(
                DragGesture()
                    .updating($isDragging) { _, isDragging, _ in
                        isDragging = true
                    }
                    .onChanged(onDragChange(_:))
                    .onEnded(onDragEnded(_:))
            )
            .onChange(of: gestureState) { state in
                guard state == .started else { return }
                gestureState = .active
            }
            .onChange(of: isDragging) { value in
                if value, gestureState != .started {
                    gestureState = .started
                } else if !value, gestureState != .ended {
                    gestureState = .cancelled
                }
            }
    }

    func onDragChange(_ value: DragGesture.Value) {
        guard gestureState == .started || gestureState == .active else { return }
        onUpdate(value)
    }

    func onDragEnded(_ value: DragGesture.Value) {
        gestureState = .ended
        onEnd(value)
    }

    enum GestureStatus: Equatable {
        case idle
        case started
        case active
        case ended
        case cancelled
    }
}

struct FileListView_Previews: PreviewProvider {
    static var previews: some View {
        NavigationView {
            FileListView()
                .mockDI()
        }
    }
}
