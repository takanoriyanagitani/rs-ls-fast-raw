use std::ffi::CString;

use std::io;

use std::io::BufWriter;
use std::io::Write;

use libc::dirent;
use libc::DIR;

pub fn dirent2name(d: &dirent) -> Vec<u8> {
    let namlen: u16 = d.d_namlen;
    let len: usize = namlen.into();

    let d_name: [i8; 1024] = d.d_name;
    let iptr: *const i8 = d_name.as_ptr();
    let uptr: *const u8 = iptr as *const u8;

    #[allow(unsafe_code)]
    let dname: &[u8] = unsafe { std::slice::from_raw_parts(uptr, len) };
    dname.into()
}

#[derive(Debug)]
pub struct RawDir {
    dirfd: i32,
}

impl Drop for RawDir {
    fn drop(&mut self) {
        #[allow(unsafe_code)]
        unsafe {
            libc::close(self.dirfd)
        };
    }
}

impl RawDir {
    fn from_raw(dirfd: i32, pathname: *const i8) -> Result<Self, io::Error> {
        let flags: i32 = libc::O_RDONLY | libc::O_DIRECTORY;

        #[allow(unsafe_code)]
        let fd: i32 = unsafe { libc::openat(dirfd, pathname, flags) };

        if fd < 0 {
            return Err(io::Error::other("unable to open"));
        }

        Ok(Self { dirfd: fd })
    }

    pub fn from_current(current: &str) -> Result<Self, io::Error> {
        let cs: CString = CString::new(current).map_err(io::Error::other)?;
        let pi: *const i8 = cs.as_ptr();
        Self::from_raw(libc::AT_FDCWD, pi)
    }
}

#[derive(Debug)]
pub struct DirReader {
    pdir: *mut DIR,
    raw: RawDir,
}

impl Drop for DirReader {
    fn drop(&mut self) {
        #[allow(unsafe_code)]
        unsafe {
            libc::closedir(self.pdir)
        };
    }
}

impl DirReader {
    pub fn from_fd(raw: RawDir, fd: i32) -> Result<Self, io::Error> {
        #[allow(unsafe_code)]
        let pdir: *mut DIR = unsafe { libc::fdopendir(fd) };
        match pdir.is_null() {
            true => Err(io::Error::other("invalid fd")),
            false => Ok(Self { pdir, raw }),
        }
    }

    pub fn from_raw(raw: RawDir) -> Result<Self, io::Error> {
        let fd: i32 = raw.dirfd;
        Self::from_fd(raw, fd)
    }

    pub fn raw_dir(&self) -> &RawDir {
        &self.raw
    }
}

impl Iterator for DirReader {
    type Item = Result<Vec<u8>, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        #[allow(unsafe_code)]
        let d: *const dirent = unsafe { libc::readdir(self.pdir) };

        #[allow(unsafe_code)]
        let so: Option<&dirent> = unsafe { d.as_ref() };

        so.map(|d: &dirent| Ok(dirent2name(d)))
    }
}

pub fn dirname2dirent_names(
    dirname: &str,
) -> Result<impl Iterator<Item = Result<Vec<u8>, io::Error>>, io::Error> {
    let raw: RawDir = RawDir::from_current(dirname)?;
    let dr: DirReader = DirReader::from_raw(raw)?;
    Ok(dr)
}

pub fn basename_writer_new<W>(mut writer: W) -> impl FnMut(Vec<u8>) -> Result<(), io::Error>
where
    W: Write,
{
    move |basename: Vec<u8>| {
        writer.write_all(&basename)?;
        writer.write_all(b"\n")?;
        writer.flush()
    }
}

pub fn basename_writer_stdout() -> impl FnMut(Vec<u8>) -> Result<(), io::Error> {
    let o = io::stdout();
    let ol = o.lock();
    let bw = BufWriter::new(ol);
    basename_writer_new(bw)
}

pub fn dir2dirents2writer<W>(dirname: &str, mut writer: W) -> Result<(), io::Error>
where
    W: FnMut(Vec<u8>) -> Result<(), io::Error>,
{
    let basenames = dirname2dirent_names(dirname)?;
    for rslt in basenames {
        let basename: Vec<u8> = rslt?;
        writer(basename)?;
    }
    Ok(())
}

pub fn dir2dirents2stdout(dirname: &str) -> Result<(), io::Error> {
    dir2dirents2writer(dirname, basename_writer_stdout())
}

pub fn arg2dir2dirents2stdout() -> Result<(), io::Error> {
    let mut args = std::env::args();
    args.next(); // 1st arg is the name of the executable
    let s: String = args.next().unwrap_or_else(|| ".".into());
    dir2dirents2stdout(s.as_str())
}
