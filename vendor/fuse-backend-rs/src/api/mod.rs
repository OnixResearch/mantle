// Copyright (C) 2020 Alibaba Cloud. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

//! Fuse Application Programming Interfaces(API).
//!
//! The Fuse application programming interfaces(API) layer is an intermediate layer
//! between the transport layer and the backend file system drivers. It provides:
//! - [struct Server](server/struct.Server.html) to receive requests from/send reply to the
//!   transport layer.
//! - [trait FileSystem](filesystem/trait.FileSystem.html) for backend file system drivers to
//!   implement fs operations.
//! - [struct Vfs](vfs/struct.Vfs.html), a simple union file system to help organize multiple
//!   backend file systems.

mod pseudo_fs;

pub mod vfs;
pub use vfs::validate_path_component;
pub use vfs::BackFileSystem;
pub use vfs::BackendFileSystem;
pub use vfs::Vfs;
pub use vfs::VfsIndex;
pub use vfs::VfsOptions;
pub use vfs::CURRENT_DIR_CSTR;
pub use vfs::EMPTY_CSTR;
pub use vfs::PARENT_DIR_CSTR;
pub use vfs::PROC_SELF_FD_CSTR;
pub use vfs::SLASH_ASCII;
pub use vfs::VFS_MAX_INO;

pub mod filesystem;
pub mod server;
