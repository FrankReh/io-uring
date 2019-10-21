use std::{ io, ptr, mem };
use std::sync::atomic;
use std::convert::TryFrom;
use std::os::unix::io::{ AsRawFd, IntoRawFd, FromRawFd, RawFd };


#[doc(hidden)]
#[macro_export]
macro_rules! mmap_offset {
    ( $mmap:ident + $offset:expr => $ty:ty ) => {
        $mmap.as_mut_ptr().add($offset as _) as $ty
    };
    ( unsafe $( let $val:ident = $mmap:ident + $offset:expr => $ty:ty );+ $(;)? ) => {
        $(
            let $val = unsafe { mmap_offset!($mmap + $offset => $ty) };
        )*
    }
}


pub struct Mmap {
    addr: *mut libc::c_void,
    len: usize
}

impl Mmap {
    pub fn new(fd: &Fd, offset: i64, len: usize) -> io::Result<Mmap> {
        unsafe {
            match libc::mmap(
                ptr::null_mut(),
                len,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_POPULATE,
                fd.as_raw_fd(),
                offset
            ) {
                libc::MAP_FAILED => Err(io::Error::last_os_error()),
                addr => Ok(Mmap { addr, len })
            }
        }
    }

    pub fn as_mut_ptr(&self) -> *mut libc::c_void {
        self.addr
    }
}

impl Drop for Mmap {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.addr, self.len);

            // TODO log fail
        }
    }
}

pub struct Fd(RawFd);

impl TryFrom<RawFd> for Fd {
    type Error = ();

    fn try_from(value: RawFd) -> Result<Fd, Self::Error> {
        if value >= 0 {
            Ok(Fd(value))
        } else {
            Err(())
        }
    }
}

impl AsRawFd for Fd {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl IntoRawFd for Fd {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.0;
        mem::forget(self);
        fd
    }
}

impl FromRawFd for Fd {
    unsafe fn from_raw_fd(fd: RawFd) -> Fd {
        Fd(fd)
    }
}

impl Drop for Fd {
    fn drop(&mut self) {
        unsafe {
            libc::close(self.0);
        }
    }
}

pub unsafe fn unsync_load(u: *const atomic::AtomicU32) -> u32 {
    *(u as *const u32)
}