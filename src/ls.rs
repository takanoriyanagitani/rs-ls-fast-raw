#[cfg(any(doc, target_os = "macos"))]
pub mod darwin;

#[cfg(any(doc, target_os = "linux"))]
pub mod linux_direct;
