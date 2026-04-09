// Copyright (C) 2023 Ant Group. All rights reserved.
// SPDX-License-Identifier: Apache-2.0

use std::ffi::CString;
use std::io::Error;
use std::io::Result;
use std::io::{self};

use crate::abi::fuse_abi::stat64;

pub(super) fn is_dir(st: stat64) -> bool {
    st.st_mode & libc::S_IFMT == libc::S_IFDIR
}

pub(super) fn to_cstring(name: &str) -> Result<CString> {
    CString::new(name).map_err(|e| Error::new(io::ErrorKind::InvalidData, e))
}
