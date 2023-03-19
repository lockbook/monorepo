# Technical Design
These docs provide an overview of Lockbook's design for the Lockbook team, community contributors, and interested users.

## Requirements
Lockbook is designed to provide trustless privacy in a note-taking app that users will love. We have an ambitious feature set to distinguish Lockbook from competitors. We build for extensibility so our lean engineering team can focus on core issues, leaving integrations and other nice-to-haves to the community.

Features:
- Register & Login
    - Users' identities are represented by cryptographic key pairs.
    - Users authenticate using self-managed private keys, which are used to encrypt files before they leave users' devices.
    - Users register by generating a key pair and registering their public key with a username on Lockbook's server.
    - Users log in by copying their private key to a new device via QR code or account string. Lockbook cannot recover lost private keys.
- Upgrade To Premium
    - Lockbook is free to use while the total size of a user's files on Lockbook's server is below some threshold.
    - Lockbook charges for file storage beyond the free tier and profits by marking up the cost of storage.
    - Payments are processed through the App Store, Play Store, or Stripe. Support for paying with cryptocurrencies is planned.
    - Files are compressed and encrypted by users' devices, so users are billed based on the size of files after compression and encryption.
    - Apps provide features to understand and manage storage usage, including warnings when users approach their storage limit.
- Edit & Sync Files
    - Apps allow users to create, edit, organize, and delete files. Files are organized into folders, forming a file tree.
    - Apps store files on users' devices for offline access and editing.
    - When a device has internet access, it can push changes to the server and pull changes that have been pushed from other devices.
    - Apps make a best effort to reconcile concurrent edits to the same files on multiple devices.
    - Documents are formatted using Markdown for seamless compatibility with plaintext editors.
    - Apps other than the CLI also support drawings in a custom, cross-platform file format.
- Import & Export To Device
    - Users can import files from their device's filesystem into Lockbook.
    - Users can export some or all files from Lockbook into their device's filesystem, which facilitates backups.
    - Support for mounting Lockbook as a FUSE (File System in User Space) is planned.
- Search Files
    - Users can search their files by name or content.
    - Search happens client-side because only users' devices know the names and contents of their documents.
    - Apps raise frequently used or otherwise recommended files for easy access.
    - Support for selecting favorite files for easy access is planned.
- Share Files
    - Users can share and un-share documents or folders; sharing a folder grants access to all files in the folder.
    - Users can organize files shared with them into their file tree and rename them without affecting their appearance for other users.
    - Shares can be performed with read or write access.

## Architecture
Lockbook's server is a single Rust process running on an AWS EC2 instance. Account and file metadata are stored in a custom in-process database, [db-rs](https://github.com/Parth/db-rs). File contents, which are compressed and encrypted before reaching the server, are stored on a mounted drive. While running a single process poses limitations for reliability and scale, some of this is mitigated by the correctness and performance characteristics of Rust as a language, and Lockbook's emphasis on a quality offline experience makes transient server unavailability tolerable. Until server load warrants re-architecting, engineering effort is better spent perfecting the user experience.

Lockbook's apps are native apps written in Swift for Apple, Kotlin for Android, and Rust for other clients. Native apps provide superior performance, stability, and energy efficiency compared to portable Javascript apps and allow us to use platform-specific APIs like PencilKit for a smooth drawing experience. This comes at the expense of code reuse, which we mitigate using a Rust library, Lockbook Core, that is compiled into our apps. Core provides the API for common Lockbook operations and manages registration & login, encryption, file syncing, import & export, search, and more. We also reuse code between Core and Server for data models and file tree analysis.

## Development & Operations
Lockbook engineers collaborate in our [Discord channel](https://discord.gg/kWgyhH3Ztu) and all source code lives in our [GitHub monorepo](https://github.com/lockbook/). We use GitHub Issues for issue tracking and GitHub Actions for continuous testing, which is executed on a dedicated machine in the engineering team's possession. Releases are performed using [Releaser](https://github.com/lockbook/lockbook/tree/master/utils/releaser), a Rust program that manages our many release channels, for which the necessary secrets live on a dedicated machine in the engineering team's possession. Maintenance and incident resolution are performed using our [Admin CLI](https://github.com/lockbook/lockbook/tree/master/clients/admin), which authenticates access by validating the signature of a self-managed private key (for users configured to have access on our server). Other development tools can be found in our [utils folder](https://github.com/lockbook/lockbook/tree/master/utils).

## Further Reading
See these additional resources for a deeper look at our engineering challenges and solutions:
- [Data Model](./data_model.md)
- [Sync](./sync.md)
- [Sharing](./sharing.md)
- [Billing](./billing.md)
