import Foundation

public struct DecryptedFileMetadata: Codable, Identifiable, Equatable, Hashable, Comparable {
    public var fileType: FileType

    public var id: UUID
    public var parent: UUID
    public var decryptedName: String
    public var owner: String
    public var contentVersion: UInt64
    public var metadataVersion: UInt64
    public var isRoot: Bool { parent == id }
    public static func == (lhs: DecryptedFileMetadata, rhs: DecryptedFileMetadata) -> Bool {
        return lhs.fileType == rhs.fileType &&
            lhs.id == rhs.id &&
//            lhs.metadataVersion == rhs.metadataVersion && // TODO don't do this here, do this at the view instead
//            lhs.contentVersion == rhs.contentVersion &&
            lhs.parent == rhs.parent &&
            lhs.owner == rhs.owner &&
            lhs.decryptedName == rhs.decryptedName
    }

    public func hash(into hasher: inout Hasher) {
        print("hash called \(decryptedName)")
        hasher.combine(id)
//        hasher.combine(decryptedName)
    }

    public static func <(lhs: DecryptedFileMetadata, rhs: DecryptedFileMetadata) -> Bool {
        // If the types are different, group folders higher
        if lhs.fileType == .Folder && rhs.fileType == .Document {
            return true
        }

        if rhs.fileType == .Folder && lhs.fileType == .Document {
            return false
        }

        // Otherwise sort alphabetically
        return lhs.decryptedName < rhs.decryptedName
    }
}

public enum FileType: String, Codable {
    case Document
    case Folder
}

public struct FolderAccessInfo: Codable {
    var folderId: UUID
    var accessKey: EncryptedValueWithNonce
}

public struct UserAccessInfo: Codable {
    var username: Account.Username
    var encryptedBy: String
    var accessKey: EncryptedValue
}

public struct SignedValue: Codable {
    var content: String
    var signature: String
}

public struct EncryptedValueWithNonce: Codable {
    var value: [UInt]
    var nonce: [UInt]
}

public struct EncryptedValue: Codable {
    var value: [UInt]
}
