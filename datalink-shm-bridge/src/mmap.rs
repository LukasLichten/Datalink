// shm-bridge
//
// Copyright (c) 2014 Jared Stafford (jspenguin@jspenguin.org)
// Copyright (c) 2024 Damir Jelić
// Copyright (c) 2024 Lukas Lichten
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::{fs::{self, File}, os::windows::{fs::OpenOptionsExt,prelude::AsRawHandle}, path::PathBuf, str::FromStr};
use windows::{
    core::HSTRING,
    Wdk::System::SystemServices::PAGE_READWRITE,
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::Memory::{CreateFileMappingW, PAGE_PROTECTION_FLAGS},
        Storage::FileSystem::FILE_ATTRIBUTE_TEMPORARY
    },
};

const TMPFS_MOUNT:&str = "/dev/shm";

pub fn get_tmpfs_mountpoint() -> Option<PathBuf> {
    let buf = PathBuf::from_str(
        // Technically, this is infalliable on Some() input, but if that function is changed it
        // could, and we have to wrap error handling for PathBuf anyway
        crate::convert_linux_path_to_wine(Some(TMPFS_MOUNT.to_string()))?.as_str()
    ).ok()?;

    if buf.exists() {
        Some(buf)
    } else {
        None
    }
}

/// File-backed named shared memory[1].
///
/// This will create named shared memory backed by a file.
///
/// The shared memory will be kept alive as long as the [`FileMapping`] object
/// is allive, dropping it will free the shared memory.
///
/// The [`FileMapping`] uses the Windows [`CreateFileMappingW`] function
/// underneath.
///
/// [1]: https://learn.microsoft.com/en-us/windows/win32/memory/creating-named-shared-memory
pub struct FileMapping {
    handle: HANDLE,
    path: PathBuf
}

impl FileMapping {
    /// Create a new [`FileMapping`], the given file will be used as the backing
    /// storage.
    ///
    /// # Arguments
    ///
    /// * `name` - The name the [`FileMapping`] should have, other Windows
    ///   applications can open the shared memory using this name.
    ///
    /// * `file` - The file that should be used as the backing storage of the
    ///   [`FileMapping`]. The file will be resized to have the correct length.
    ///
    /// * `size` - The desiered size the [`FileMapping`] should have, i.e. the
    ///   number of bytes the [`FileMapping`] should have.
    pub fn new(name: &str, file: &File, size: usize, path: PathBuf) -> Result<Self,String> {
        // Ensure the file is of the correct size.
        file.set_len(size as u64).map_err(|e| format!("Couldn't set the file size of the FileMapping: {e}"))?;

        let high_size: u32 = ((size as u64 & 0xFFFF_FFFF_0000_0000_u64) >> 32) as u32;
        let low_size: u32 = (size as u64 & 0xFFFF_FFFF_u64) as u32;

        // Windows uses UTF-16, so we need to convert the UTF-8 based Rust string
        // accordingly.
        let name = HSTRING::from(name);
        let handle = HANDLE(file.as_raw_handle() as _);

        let handle = unsafe {
            CreateFileMappingW(
                handle,
                None,
                PAGE_PROTECTION_FLAGS(PAGE_READWRITE),
                high_size,
                low_size,
                &name,
            )
        };

        match handle {
            Ok(handle) => Ok(FileMapping { handle, path }),
            Err(e) => Err(format!("Failed to create the FileMapping: {e}")),
        }
    }
}

impl Drop for FileMapping {
    fn drop(&mut self) {
        // There's not much we can do if an error happens here, so let's ignore it.
        let _ = unsafe { CloseHandle(self.handle) };
        let _ = fs::remove_file(self.path.as_path());
    }
}

pub(crate) fn create_file_mapping(mut dir: PathBuf, file_name: &str, size: usize) -> Result<FileMapping,String> {
    dir.push(file_name);
    let path = dir;

    // First we create a /dev/shm backed file.
    //
    // Now hear me out, usually we should use `shm_open(3)` here, but on Linux
    // `shm_open()` just calls `open()`. It does have some logic to find the
    // tmpfs location if it's mounted in a non-standard location. Since we can't
    // call `shm_open(3)` from inside the Wine environment
    let file = File::options()
        .read(true)
        .write(true)
        .attributes(FILE_ATTRIBUTE_TEMPORARY.0)
        .create(true)
        .open(path.as_path())
        .map_err(|_| format!("Could not open the tmpfs file: {path:?}"))?;

    // Now we create a mapping that is backed by the previously created /dev/shm`
    // file.
    let mapping = FileMapping::new(
        // We're going to use the same names the Simulator uses. This ensures that the
        // simulator will reuse this `/dev/shm` backed mapping instead of creating a new anonymous
        // one. Making the simulator reuse the mapping in turn means that the telemetry data will
        // be available in `/dev/shm` as well, making it accessible to Linux.
        file_name,
        // Pass in the handle of the `/dev/shm` file, this ensures that the file mapping is a file
        // backed one and is using our tmpfs file created on the Linux side.
        &file,
        // The documentation[1] for CreateFileMapping states that the sizes are only necessary if
        // we're using a `INVALID_HANDLE_VALUE` for the file handle.
        //
        // It also states the following:
        // > If this parameter and dwMaximumSizeHigh are 0 (zero), the maximum size of the
        // > file mapping object is equal to the current size of the file that hFile identifies.
        //
        // This sadly doesn't seem to work with our `/dev/shm` file and makes the Simulator crash,
        // so we're passing the sizes manually.
        //
        // [1]: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-createfilemappinga#parameters
        size,

        path
    )?;

    // Return the mapping, the caller needs to ensure that the mapping object stays
    // alive. On the other hand, the `/dev/shm` backed file can be closed.
    Ok(mapping)
}

