import Foundation
import SwiftUI
import SwiftLockbookCore

struct PendingSharesView: View {
    
    @EnvironmentObject var settings: SettingsService
    
    var body: some View {
        if settings.pendingShares.isEmpty {
            noPendingShares
        } else {
            pendingShares
        }
        
    }
    
    
    @ViewBuilder
    var pendingShares: some View {
        VStack {
            List(settings.pendingShares) { meta in
                SharedFileCell(meta: meta)
            }
            
            Spacer()
        }
        .navigationBarTitle("Pending Shares")
        .onAppear {
                settings.calculatePendingShares()
            }
    }
    
    @ViewBuilder
    var noPendingShares: some View {
        VStack {
            Spacer()
            Image(systemName: "shared.with.you.slash")
                .padding(.vertical, 5)
                .imageScale(.large)
            Text("You have no pending shares.")
            Spacer()
        }
        .navigationBarTitle("Pending Shares")
        .onAppear {
                settings.calculatePendingShares()
            }

    }
    
}

struct SharedFileCell: View {
    
    @EnvironmentObject var settings: SettingsService
    @EnvironmentObject var sheets: SheetState
    
    let meta: File

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: meta.fileType == .Folder ? "folder" : "doc")
                .foregroundColor(meta.fileType == .Folder ? .blue : .secondary)
                
            Text(meta.name)
                .font(.title3)
                
            Spacer()
            
            
            Button {
                sheets.acceptingShareInfo = meta
            } label: {
                Image(systemName: "plus.circle")
                    .imageScale(.large)
                    .foregroundColor(.blue)
            }
            
            Button {
                settings.rejectShare(id: meta.id)
            } label: {
                Image(systemName: "minus.circle")
                    .imageScale(.large)
                    .foregroundColor(.red)
            }
        }
                .padding(.vertical, 10)
                .contentShape(Rectangle())
                .sheet(isPresented: $sheets.acceptingShare) {
                    AcceptShareSheet()
                }
    }
}

