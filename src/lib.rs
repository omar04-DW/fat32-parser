#![cfg_attr(not(test), no_std)]

pub mod allocator;
pub mod block_device;
pub mod boot_sector;
pub mod fat;
pub mod dir_entry;
pub mod filesystem;
pub mod error;

pub use filesystem::Fat32Fs;
pub use block_device::{BlockDevice, BlockDeviceError};
pub use error::{Fat32Error, Result};