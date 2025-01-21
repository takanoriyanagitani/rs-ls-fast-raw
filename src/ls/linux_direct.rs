use std::io;
use std::io::BufWriter;
use std::io::Write;

use std::ffi::CStr;
use std::ffi::CString;

use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::fd::RawFd;

use libc::c_char;
use libc::c_int;
use libc::c_long;
use libc::c_ushort;
use libc::c_void;

use libc::syscall;
use libc::SYS_getdents64;

use linux_raw_sys::general::linux_dirent64;

fn getdents64(fd: c_int, dirp: *mut c_void, count: usize) -> c_long {
    #[allow(unsafe_code)]
    unsafe {
        syscall(SYS_getdents64, fd, dirp, count)
    }
}

pub fn dirent2writer<W>(d: &linux_dirent64, wtr: &mut W) -> Result<(), io::Error>
where
    W: Write,
{
    let name = &d.d_name; // null-terminated
    let nptr: *const c_char = name.as_ptr();
    let iptr: *const i8 = nptr;

    #[allow(unsafe_code)]
    let cstr: &CStr = unsafe { CStr::from_ptr(iptr) };
    let s: &[u8] = cstr.to_bytes();

    wtr.write_all(s)?;
    wtr.write_all(b"\n")
}

pub fn fd2dirents2writer<W>(fd: c_int, mut wtr: W) -> Result<(), io::Error>
where
    W: FnMut(&linux_dirent64) -> Result<(), io::Error>,
{
    const BUFSIZ: usize = 2048;
    let mut buf: [u8; BUFSIZ] = [0; BUFSIZ];
    loop {
        let mptr: *mut u8 = buf.as_mut_ptr();
        let mv: *mut c_void = mptr as *mut c_void;

        let nread: c_long = getdents64(fd, mv, BUFSIZ);
        if -1 == nread {
            return Err(io::Error::other("unable to read dir(getdents64)"));
        }
        if 0 == nread {
            return Ok(());
        }

        let mut bpos: c_long = 0;
        let ptr: *const u8 = mptr;
        while bpos < nread {
            let optr: *const u8 = ptr.wrapping_add(bpos as usize);
            let pdirent: *const linux_dirent64 = optr as *const linux_dirent64;
            #[allow(unsafe_code)]
            let odirent: Option<&linux_dirent64> = unsafe { pdirent.as_ref() };
            match odirent {
                None => return Ok(()),
                Some(d) => {
                    wtr(d)?;
                    let len: c_ushort = d.d_reclen;
                    bpos += i64::from(len);
                }
            }
        }
    }
}

pub fn owned_fd2dirents2writer<W>(fd: OwnedFd, wtr: W) -> Result<(), io::Error>
where
    W: FnMut(&linux_dirent64) -> Result<(), io::Error>,
{
    let raw: RawFd = fd.as_raw_fd();
    fd2dirents2writer(raw, wtr)
}

fn dirfd2dirents2writer<W>(dirfd: i32, fullpath: *const i8, wtr: W) -> Result<(), io::Error>
where
    W: FnMut(&linux_dirent64) -> Result<(), io::Error>,
{
    let flags: i32 = libc::O_RDONLY | libc::O_DIRECTORY;

    #[allow(unsafe_code)]
    let fd: i32 = unsafe { libc::openat(dirfd, fullpath, flags) };
    if fd < 0 {
        return Err(io::Error::other("unable to open"));
    }

    #[allow(unsafe_code)]
    let owned: OwnedFd = unsafe { OwnedFd::from_raw_fd(fd) };
    owned_fd2dirents2writer(owned, wtr)
}

pub fn current2dirents2writer<W>(fullpath: String, wtr: W) -> Result<(), io::Error>
where
    W: FnMut(&linux_dirent64) -> Result<(), io::Error>,
{
    let c: CString = CString::new(fullpath).map_err(io::Error::other)?;
    let cstr: &CStr = &c;
    let ptr: *const i8 = cstr.as_ptr();
    dirfd2dirents2writer(libc::AT_FDCWD, ptr, wtr)
}

pub fn dirname2dirents2writer<W>(dirname: String, mut wtr: W) -> Result<(), io::Error>
where
    W: Write,
{
    let writer = |dirent: &linux_dirent64| {
        dirent2writer(dirent, &mut wtr)?;
        Ok(())
    };
    current2dirents2writer(dirname, writer)
}

pub fn dirname2dirents2stdout(dirname: String) -> Result<(), io::Error> {
    let o = io::stdout();
    let mut ol = o.lock();
    {
        let mut bw = BufWriter::new(&mut ol);
        dirname2dirents2writer(dirname, &mut bw)?;
        bw.flush()?;
    }
    ol.flush()
}

pub fn arg2dir2dirents2stdout() -> Result<(), io::Error> {
    let mut args = std::env::args();
    args.next(); // 1st arg is the name of the executable
    let s: String = args.next().unwrap_or_else(|| ".".into());
    dirname2dirents2stdout(s)
}
