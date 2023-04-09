use crate::prelude::*;

#[cold]
fn format_non_errno(e: Error) -> CowStr<'static> {
    CowStr::Owned(e.to_string().into())
}

#[cold]
fn format_unknown_errno(errno: libc::c_int) -> Box<str> {
    let mut s = String::new();
    s.push_str("EUNKNOWN: Unknown errno ");
    let mut errno = reinterpret_i32_u32(errno);
    while errno >= 10 {
        s.push(char::from_digit(errno % 10, 10).unwrap());
        errno /= 10;
    }
    s.push(char::from_digit(errno, 10).unwrap());
    s.into()
}

fn format_errno(errno: libc::c_int) -> CowStr<'static> {
    // It's okay if some of them are unreachable, as errno numbers differ across architectures
    // and are sometimes aliased.
    #![allow(unreachable_patterns)]

    // List taken from glibc, but with stuff not supported by Rust dropped.
    // IMPORTANT: Keep in sync with `arbitrary_errno`. Ideally, this would use a proc macro or
    // script, but I'm too lazy to do that for such a one-off thing that doesn't change often in
    // practice.
    match errno {
        libc::E2BIG => CowStr::Borrowed("E2BIG: Argument list too long"),
        libc::EACCES => CowStr::Borrowed("EACCES: Permission denied"),
        libc::EADDRINUSE => CowStr::Borrowed("EADDRINUSE: Address already in use"),
        libc::EADDRNOTAVAIL => CowStr::Borrowed("EADDRNOTAVAIL: Cannot assign requested address"),
        libc::EADV => CowStr::Borrowed("EADV: Advertise error"),
        libc::EAFNOSUPPORT => {
            CowStr::Borrowed("EAFNOSUPPORT: Address family not supported by protocol")
        }
        libc::EAGAIN => CowStr::Borrowed("EAGAIN: Resource temporarily unavailable"),
        libc::EALREADY => CowStr::Borrowed("EALREADY: Operation already in progress"),
        libc::EBADE => CowStr::Borrowed("EBADE: Invalid exchange"),
        libc::EBADF => CowStr::Borrowed("EBADF: Bad file descriptor"),
        libc::EBADFD => CowStr::Borrowed("EBADFD: File descriptor in bad state"),
        libc::EBADMSG => CowStr::Borrowed("EBADMSG: Bad message"),
        libc::EBADR => CowStr::Borrowed("EBADR: Invalid request descriptor"),
        libc::EBADRQC => CowStr::Borrowed("EBADRQC: Invalid request code"),
        libc::EBADSLT => CowStr::Borrowed("EBADSLT: Invalid slot"),
        libc::EBFONT => CowStr::Borrowed("EBFONT: Bad font file format"),
        libc::EBUSY => CowStr::Borrowed("EBUSY: Device or resource busy"),
        libc::ECANCELED => CowStr::Borrowed("ECANCELED: Operation canceled"),
        libc::ECHILD => CowStr::Borrowed("ECHILD: No child processes"),
        libc::ECHRNG => CowStr::Borrowed("ECHRNG: Channel number out of range"),
        libc::ECOMM => CowStr::Borrowed("ECOMM: Communication error on send"),
        libc::ECONNABORTED => CowStr::Borrowed("ECONNABORTED: Software caused connection abort"),
        libc::ECONNREFUSED => CowStr::Borrowed("ECONNREFUSED: Connection refused"),
        libc::ECONNRESET => CowStr::Borrowed("ECONNRESET: Connection reset by peer"),
        libc::EDEADLK => CowStr::Borrowed("EDEADLK: Resource deadlock avoided"),
        libc::EDESTADDRREQ => CowStr::Borrowed("EDESTADDRREQ: Destination address required"),
        libc::EDOM => CowStr::Borrowed("EDOM: Numerical argument out of domain"),
        libc::EDOTDOT => CowStr::Borrowed("EDOTDOT: RFS specific error"),
        libc::EDQUOT => CowStr::Borrowed("EDQUOT: Disk quota exceeded"),
        libc::EEXIST => CowStr::Borrowed("EEXIST: File exists"),
        libc::EFAULT => CowStr::Borrowed("EFAULT: Bad address"),
        libc::EFBIG => CowStr::Borrowed("EFBIG: File too large"),
        libc::EHOSTDOWN => CowStr::Borrowed("EHOSTDOWN: Host is down"),
        libc::EHOSTUNREACH => CowStr::Borrowed("EHOSTUNREACH: No route to host"),
        libc::EHWPOISON => CowStr::Borrowed("EHWPOISON: Memory page has hardware error"),
        libc::EIDRM => CowStr::Borrowed("EIDRM: Identifier removed"),
        libc::EILSEQ => {
            CowStr::Borrowed("EILSEQ: Invalid or incomplete multibyte or wide character")
        }
        libc::EINPROGRESS => CowStr::Borrowed("EINPROGRESS: Operation now in progress"),
        libc::EINTR => CowStr::Borrowed("EINTR: Interrupted system call"),
        libc::EINVAL => CowStr::Borrowed("EINVAL: Invalid argument"),
        libc::EIO => CowStr::Borrowed("EIO: Input/output error"),
        libc::EISCONN => CowStr::Borrowed("EISCONN: Transport endpoint is already connected"),
        libc::EISDIR => CowStr::Borrowed("EISDIR: Is a directory"),
        libc::EISNAM => CowStr::Borrowed("EISNAM: Is a named type file"),
        libc::EKEYEXPIRED => CowStr::Borrowed("EKEYEXPIRED: Key has expired"),
        libc::EKEYREJECTED => CowStr::Borrowed("EKEYREJECTED: Key was rejected by service"),
        libc::EKEYREVOKED => CowStr::Borrowed("EKEYREVOKED: Key has been revoked"),
        libc::EL2HLT => CowStr::Borrowed("EL2HLT: Level 2 halted"),
        libc::EL2NSYNC => CowStr::Borrowed("EL2NSYNC: Level 2 not synchronized"),
        libc::EL3HLT => CowStr::Borrowed("EL3HLT: Level 3 halted"),
        libc::EL3RST => CowStr::Borrowed("EL3RST: Level 3 reset"),
        libc::ELIBACC => CowStr::Borrowed("ELIBACC: Can not access a needed shared library"),
        libc::ELIBBAD => CowStr::Borrowed("ELIBBAD: Accessing a corrupted shared library"),
        libc::ELIBEXEC => CowStr::Borrowed("ELIBEXEC: Cannot exec a shared library directly"),
        libc::ELIBMAX => {
            CowStr::Borrowed("ELIBMAX: Attempting to link in too many shared libraries")
        }
        libc::ELIBSCN => CowStr::Borrowed("ELIBSCN: .lib section in a.out corrupted"),
        libc::ELNRNG => CowStr::Borrowed("ELNRNG: Link number out of range"),
        libc::ELOOP => CowStr::Borrowed("ELOOP: Too many levels of symbolic links"),
        libc::EMEDIUMTYPE => CowStr::Borrowed("EMEDIUMTYPE: Wrong medium type"),
        libc::EMFILE => CowStr::Borrowed("EMFILE: Too many open files"),
        libc::EMLINK => CowStr::Borrowed("EMLINK: Too many links"),
        libc::EMSGSIZE => CowStr::Borrowed("EMSGSIZE: Message too long"),
        libc::EMULTIHOP => CowStr::Borrowed("EMULTIHOP: Multihop attempted"),
        libc::ENAMETOOLONG => CowStr::Borrowed("ENAMETOOLONG: File name too long"),
        libc::ENAVAIL => CowStr::Borrowed("ENAVAIL: No XENIX semaphores available"),
        libc::ENETDOWN => CowStr::Borrowed("ENETDOWN: Network is down"),
        libc::ENETRESET => CowStr::Borrowed("ENETRESET: Network dropped connection on reset"),
        libc::ENETUNREACH => CowStr::Borrowed("ENETUNREACH: Network is unreachable"),
        libc::ENFILE => CowStr::Borrowed("ENFILE: Too many open files in system"),
        libc::ENOANO => CowStr::Borrowed("ENOANO: No anode"),
        libc::ENOBUFS => CowStr::Borrowed("ENOBUFS: No buffer space available"),
        libc::ENOCSI => CowStr::Borrowed("ENOCSI: No CSI structure available"),
        libc::ENODATA => CowStr::Borrowed("ENODATA: No data available"),
        libc::ENODEV => CowStr::Borrowed("ENODEV: No such device"),
        libc::ENOENT => CowStr::Borrowed("ENOENT: No such file or directory"),
        libc::ENOEXEC => CowStr::Borrowed("ENOEXEC: Exec format error"),
        libc::ENOKEY => CowStr::Borrowed("ENOKEY: Required key not available"),
        libc::ENOLCK => CowStr::Borrowed("ENOLCK: No locks available"),
        libc::ENOLINK => CowStr::Borrowed("ENOLINK: Link has been severed"),
        libc::ENOMEDIUM => CowStr::Borrowed("ENOMEDIUM: No medium found"),
        libc::ENOMEM => CowStr::Borrowed("ENOMEM: Cannot allocate memory"),
        libc::ENOMSG => CowStr::Borrowed("ENOMSG: No message of desired type"),
        libc::ENONET => CowStr::Borrowed("ENONET: Machine is not on the network"),
        libc::ENOPKG => CowStr::Borrowed("ENOPKG: Package not installed"),
        libc::ENOPROTOOPT => CowStr::Borrowed("ENOPROTOOPT: Protocol not available"),
        libc::ENOSPC => CowStr::Borrowed("ENOSPC: No space left on device"),
        libc::ENOSR => CowStr::Borrowed("ENOSR: Out of streams resources"),
        libc::ENOSTR => CowStr::Borrowed("ENOSTR: Device not a stream"),
        libc::ENOSYS => CowStr::Borrowed("ENOSYS: Function not implemented"),
        libc::ENOTBLK => CowStr::Borrowed("ENOTBLK: Block device required"),
        libc::ENOTCONN => CowStr::Borrowed("ENOTCONN: Transport endpoint is not connected"),
        libc::ENOTDIR => CowStr::Borrowed("ENOTDIR: Not a directory"),
        libc::ENOTEMPTY => CowStr::Borrowed("ENOTEMPTY: Directory not empty"),
        libc::ENOTNAM => CowStr::Borrowed("ENOTNAM: Not a XENIX named type file"),
        libc::ENOTRECOVERABLE => CowStr::Borrowed("ENOTRECOVERABLE: State not recoverable"),
        libc::ENOTSOCK => CowStr::Borrowed("ENOTSOCK: Socket operation on non-socket"),
        libc::ENOTSUP => CowStr::Borrowed("ENOTSUP: Not supported"),
        libc::ENOTTY => CowStr::Borrowed("ENOTTY: Inappropriate ioctl for device"),
        libc::ENOTUNIQ => CowStr::Borrowed("ENOTUNIQ: Name not unique on network"),
        libc::ENXIO => CowStr::Borrowed("ENXIO: No such device or address"),
        libc::EOPNOTSUPP => CowStr::Borrowed("EOPNOTSUPP: Operation not supported"),
        libc::EOVERFLOW => CowStr::Borrowed("EOVERFLOW: Value too large for defined data type"),
        libc::EOWNERDEAD => CowStr::Borrowed("EOWNERDEAD: Owner died"),
        libc::EPERM => CowStr::Borrowed("EPERM: Operation not permitted"),
        libc::EPFNOSUPPORT => CowStr::Borrowed("EPFNOSUPPORT: Protocol family not supported"),
        libc::EPIPE => CowStr::Borrowed("EPIPE: Broken pipe"),
        libc::EPROTO => CowStr::Borrowed("EPROTO: Protocol error"),
        libc::EPROTONOSUPPORT => CowStr::Borrowed("EPROTONOSUPPORT: Protocol not supported"),
        libc::EPROTOTYPE => CowStr::Borrowed("EPROTOTYPE: Protocol wrong type for socket"),
        libc::ERANGE => CowStr::Borrowed("ERANGE: Numerical result out of range"),
        libc::EREMCHG => CowStr::Borrowed("EREMCHG: Remote address changed"),
        libc::EREMOTE => CowStr::Borrowed("EREMOTE: Object is remote"),
        libc::EREMOTEIO => CowStr::Borrowed("EREMOTEIO: Remote I/O error"),
        libc::ERESTART => CowStr::Borrowed("ERESTART: Interrupted system call should be restarted"),
        libc::ERFKILL => CowStr::Borrowed("ERFKILL: Operation not possible due to RF-kill"),
        libc::EROFS => CowStr::Borrowed("EROFS: Read-only file system"),
        libc::ESHUTDOWN => {
            CowStr::Borrowed("ESHUTDOWN: Cannot send after transport endpoint shutdown")
        }
        libc::ESOCKTNOSUPPORT => CowStr::Borrowed("ESOCKTNOSUPPORT: Socket type not supported"),
        libc::ESPIPE => CowStr::Borrowed("ESPIPE: Illegal seek"),
        libc::ESRCH => CowStr::Borrowed("ESRCH: No such process"),
        libc::ESRMNT => CowStr::Borrowed("ESRMNT: Srmount error"),
        libc::ESTALE => CowStr::Borrowed("ESTALE: Stale file handle"),
        libc::ESTRPIPE => CowStr::Borrowed("ESTRPIPE: Streams pipe error"),
        libc::ETIME => CowStr::Borrowed("ETIME: Timer expired"),
        libc::ETIMEDOUT => CowStr::Borrowed("ETIMEDOUT: Connection timed out"),
        libc::ETOOMANYREFS => CowStr::Borrowed("ETOOMANYREFS: Too many references: cannot splice"),
        libc::ETXTBSY => CowStr::Borrowed("ETXTBSY: Text file busy"),
        libc::EUCLEAN => CowStr::Borrowed("EUCLEAN: Structure needs cleaning"),
        libc::EUNATCH => CowStr::Borrowed("EUNATCH: Protocol driver not attached"),
        libc::EUSERS => CowStr::Borrowed("EUSERS: Too many users"),
        libc::EWOULDBLOCK => CowStr::Borrowed("EWOULDBLOCK: Operation would block"),
        libc::EXDEV => CowStr::Borrowed("EXDEV: Invalid cross-device link"),
        libc::EXFULL => CowStr::Borrowed("EXFULL: Exchange full"),
        _ => CowStr::Owned(format_unknown_errno(errno)),
    }
}

#[cfg(test)]
static ERRNO_LIST: &[libc::c_int] = &[
    libc::EPERM,
    libc::ENOENT,
    libc::ESRCH,
    libc::EINTR,
    libc::EIO,
    libc::ENXIO,
    libc::E2BIG,
    libc::ENOEXEC,
    libc::EBADF,
    libc::ECHILD,
    libc::EDEADLK,
    libc::ENOMEM,
    libc::EACCES,
    libc::EFAULT,
    libc::ENOTBLK,
    libc::EBUSY,
    libc::EEXIST,
    libc::EXDEV,
    libc::ENODEV,
    libc::ENOTDIR,
    libc::EISDIR,
    libc::EINVAL,
    libc::EMFILE,
    libc::ENFILE,
    libc::ENOTTY,
    libc::ETXTBSY,
    libc::EFBIG,
    libc::ENOSPC,
    libc::ESPIPE,
    libc::EROFS,
    libc::EMLINK,
    libc::EPIPE,
    libc::EDOM,
    libc::ERANGE,
    libc::EAGAIN,
    libc::EINPROGRESS,
    libc::EALREADY,
    libc::ENOTSOCK,
    libc::EMSGSIZE,
    libc::EPROTOTYPE,
    libc::ENOPROTOOPT,
    libc::EPROTONOSUPPORT,
    libc::ESOCKTNOSUPPORT,
    libc::EOPNOTSUPP,
    libc::EPFNOSUPPORT,
    libc::EAFNOSUPPORT,
    libc::EADDRINUSE,
    libc::EADDRNOTAVAIL,
    libc::ENETDOWN,
    libc::ENETUNREACH,
    libc::ENETRESET,
    libc::ECONNABORTED,
    libc::ECONNRESET,
    libc::ENOBUFS,
    libc::EISCONN,
    libc::ENOTCONN,
    libc::EDESTADDRREQ,
    libc::ESHUTDOWN,
    libc::ETOOMANYREFS,
    libc::ETIMEDOUT,
    libc::ECONNREFUSED,
    libc::ELOOP,
    libc::ENAMETOOLONG,
    libc::EHOSTDOWN,
    libc::EHOSTUNREACH,
    libc::ENOTEMPTY,
    libc::EUSERS,
    libc::EDQUOT,
    libc::ESTALE,
    libc::EREMOTE,
    libc::ENOLCK,
    libc::ENOSYS,
    libc::EILSEQ,
    libc::EBADMSG,
    libc::EIDRM,
    libc::EMULTIHOP,
    libc::ENODATA,
    libc::ENOLINK,
    libc::ENOMSG,
    libc::ENOSR,
    libc::ENOSTR,
    libc::EOVERFLOW,
    libc::EPROTO,
    libc::ETIME,
    libc::ECANCELED,
    libc::EOWNERDEAD,
    libc::ENOTRECOVERABLE,
    libc::ERESTART,
    libc::ECHRNG,
    libc::EL2NSYNC,
    libc::EL3HLT,
    libc::EL3RST,
    libc::ELNRNG,
    libc::EUNATCH,
    libc::ENOCSI,
    libc::EL2HLT,
    libc::EBADE,
    libc::EBADR,
    libc::EXFULL,
    libc::ENOANO,
    libc::EBADRQC,
    libc::EBADSLT,
    libc::EBFONT,
    libc::ENONET,
    libc::ENOPKG,
    libc::EADV,
    libc::ESRMNT,
    libc::ECOMM,
    libc::EDOTDOT,
    libc::ENOTUNIQ,
    libc::EBADFD,
    libc::EREMCHG,
    libc::ELIBACC,
    libc::ELIBBAD,
    libc::ELIBSCN,
    libc::ELIBMAX,
    libc::ELIBEXEC,
    libc::ESTRPIPE,
    libc::EUCLEAN,
    libc::ENOTNAM,
    libc::ENAVAIL,
    libc::EISNAM,
    libc::EREMOTEIO,
    libc::ENOMEDIUM,
    libc::EMEDIUMTYPE,
    libc::ENOKEY,
    libc::EKEYEXPIRED,
    libc::EKEYREVOKED,
    libc::EKEYREJECTED,
    libc::ERFKILL,
    libc::EHWPOISON,
    libc::ENOTSUP,
    libc::EWOULDBLOCK,
];

#[cfg(test)]
pub fn errno_arbitrary(g: &mut Gen) -> libc::c_int {
    *g.choose(ERRNO_LIST).unwrap()
}

#[cfg(test)]
pub fn errno_shrink(code: libc::c_int) -> impl Iterator<Item = libc::c_int> {
    ERRNO_LIST.iter().copied().filter(move |c| *c < code)
}

#[cold]
#[inline(never)]
pub fn panic_errno(e: Error, syscall_name: &'static str) -> ! {
    std::panic::panic_any(normalize_errno(e, Some(syscall_name)).into_owned())
}

pub fn normalize_errno(e: Error, syscall: Option<&'static str>) -> CowStr<'static> {
    let result = match e.raw_os_error() {
        Some(errno) => format_errno(errno),
        None => format_non_errno(e),
    };

    if let Some(syscall) = syscall {
        let mut s = result.into_owned().into_string();
        s.push_str(" from syscall '");
        s.push_str(syscall);
        s.push('\'');
        CowStr::Owned(s.into())
    } else {
        result
    }
}
