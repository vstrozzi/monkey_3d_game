use crate::SharedMemory;
use std::fs::{OpenOptions};
use std::io::Write;
use std::sync::Arc;

/// Wrapper for file-based shared memory on native platforms (UNIX).
/// Location shared data structure: /data/local/tmp/monkey_shm_*
/// Used by both python.rs binding and game_node.
pub struct NativeSharedMemory {
    ptr: *mut SharedMemory,
}

// Initialize shared memory region (by creating or opening existing)
impl NativeSharedMemory {
    pub fn new(name: &str) -> std::io::Result<Self> {
        let path = std::env::temp_dir().join(format!("monkey_shm_{}", name));
        let size = std::mem::size_of::<SharedMemory>();
        
        let mut file =  OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .truncate(true)
                .open(&path)?;

        let zeroes = vec![0u8; size];
        file.write_all(&zeroes)?;
        file.sync_all()?;
        
        #[cfg(unix)]
        let ptr = unsafe {
            use std::os::unix::io::AsRawFd;
            let fd = file.as_raw_fd();
            let ptr = libc::mmap(
                std::ptr::null_mut(),
                size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                fd,
                0,
            );
            ptr as *mut SharedMemory
        };
        
        unsafe {
            std::ptr::write(ptr, SharedMemory::new());
        }


        Ok(Self {ptr})
    }

    pub fn get(&self) -> &SharedMemory {
        unsafe { &*self.ptr }
    }

    pub fn get_mut(&mut self) -> &mut SharedMemory {
        unsafe { &mut *self.ptr }
    }
}

// Drop the shared memory mapping
impl Drop for NativeSharedMemory {
    fn drop(&mut self) {
        #[cfg(unix)]
        unsafe {
            libc::munmap(
                self.ptr as *mut libc::c_void,
                std::mem::size_of::<SharedMemory>(),
            );
        }
    }
}

// Ensure we can send and share the NativeSharedMemory across threads
unsafe impl Send for NativeSharedMemory {}
unsafe impl Sync for NativeSharedMemory {}

// Share ownership of the shaed memory across threads
pub type SharedMemoryHandle = Arc<NativeSharedMemory>;

// Create or open shm, regardless it is filled clean with 0s everytime it is called
pub fn create_shared_memory(name: &str) -> std::io::Result<SharedMemoryHandle> {
    Ok(Arc::new(NativeSharedMemory::new(name)?))
}
