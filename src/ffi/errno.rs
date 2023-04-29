use crate::prelude::*;

#[cold]
fn format_non_errno(e: Error) -> Cow<'static, str> {
    Cow::Owned(e.to_string())
}

#[cold]
fn format_unknown_errno(errno: libc::c_int) -> String {
    let mut s = String::new();
    s.push_str("EUNKNOWN: Unknown errno ");
    let mut errno = reinterpret_i32_u32(errno);
    while errno >= 10 {
        s.push(char::from_digit(errno % 10, 10).unwrap());
        errno /= 10;
    }
    s.push(char::from_digit(errno, 10).unwrap());
    s
}

pub fn normalize_errno(e: Error, syscall: Option<&'static str>) -> Cow<'static, str> {
    // It's okay if some of them are unreachable, as errno numbers differ across architectures
    // and are sometimes aliased.
    #[allow(unreachable_patterns)]
    let result = match e.raw_os_error() {
        // List taken from glibc, but with stuff not supported by Rust dropped.
        // IMPORTANT: Keep in sync with `arbitrary_errno`. Ideally, this would use a proc macro or
        // script, but I'm too lazy to do that for such a one-off thing that doesn't change often in
        // practice.
        Some(libc::E2BIG) => Cow::Borrowed("E2BIG: Argument list too long"),
        Some(libc::EACCES) => Cow::Borrowed("EACCES: Permission denied"),
        Some(libc::EADDRINUSE) => Cow::Borrowed("EADDRINUSE: Address already in use"),
        Some(libc::EADDRNOTAVAIL) => {
            Cow::Borrowed("EADDRNOTAVAIL: Cannot assign requested address")
        }
        Some(libc::EADV) => Cow::Borrowed("EADV: Advertise error"),
        Some(libc::EAFNOSUPPORT) => {
            Cow::Borrowed("EAFNOSUPPORT: Address family not supported by protocol")
        }
        Some(libc::EAGAIN) => Cow::Borrowed("EAGAIN: Resource temporarily unavailable"),
        Some(libc::EALREADY) => Cow::Borrowed("EALREADY: Operation already in progress"),
        Some(libc::EBADE) => Cow::Borrowed("EBADE: Invalid exchange"),
        Some(libc::EBADF) => Cow::Borrowed("EBADF: Bad file descriptor"),
        Some(libc::EBADFD) => Cow::Borrowed("EBADFD: File descriptor in bad state"),
        Some(libc::EBADMSG) => Cow::Borrowed("EBADMSG: Bad message"),
        Some(libc::EBADR) => Cow::Borrowed("EBADR: Invalid request descriptor"),
        Some(libc::EBADRQC) => Cow::Borrowed("EBADRQC: Invalid request code"),
        Some(libc::EBADSLT) => Cow::Borrowed("EBADSLT: Invalid slot"),
        Some(libc::EBFONT) => Cow::Borrowed("EBFONT: Bad font file format"),
        Some(libc::EBUSY) => Cow::Borrowed("EBUSY: Device or resource busy"),
        Some(libc::ECANCELED) => Cow::Borrowed("ECANCELED: Operation canceled"),
        Some(libc::ECHILD) => Cow::Borrowed("ECHILD: No child processes"),
        Some(libc::ECHRNG) => Cow::Borrowed("ECHRNG: Channel number out of range"),
        Some(libc::ECOMM) => Cow::Borrowed("ECOMM: Communication error on send"),
        Some(libc::ECONNABORTED) => Cow::Borrowed("ECONNABORTED: Software caused connection abort"),
        Some(libc::ECONNREFUSED) => Cow::Borrowed("ECONNREFUSED: Connection refused"),
        Some(libc::ECONNRESET) => Cow::Borrowed("ECONNRESET: Connection reset by peer"),
        Some(libc::EDEADLK) => Cow::Borrowed("EDEADLK: Resource deadlock avoided"),
        Some(libc::EDESTADDRREQ) => Cow::Borrowed("EDESTADDRREQ: Destination address required"),
        Some(libc::EDOM) => Cow::Borrowed("EDOM: Numerical argument out of domain"),
        Some(libc::EDOTDOT) => Cow::Borrowed("EDOTDOT: RFS specific error"),
        Some(libc::EDQUOT) => Cow::Borrowed("EDQUOT: Disk quota exceeded"),
        Some(libc::EEXIST) => Cow::Borrowed("EEXIST: File exists"),
        Some(libc::EFAULT) => Cow::Borrowed("EFAULT: Bad address"),
        Some(libc::EFBIG) => Cow::Borrowed("EFBIG: File too large"),
        Some(libc::EHOSTDOWN) => Cow::Borrowed("EHOSTDOWN: Host is down"),
        Some(libc::EHOSTUNREACH) => Cow::Borrowed("EHOSTUNREACH: No route to host"),
        Some(libc::EHWPOISON) => Cow::Borrowed("EHWPOISON: Memory page has hardware error"),
        Some(libc::EIDRM) => Cow::Borrowed("EIDRM: Identifier removed"),
        Some(libc::EILSEQ) => {
            Cow::Borrowed("EILSEQ: Invalid or incomplete multibyte or wide character")
        }
        Some(libc::EINPROGRESS) => Cow::Borrowed("EINPROGRESS: Operation now in progress"),
        Some(libc::EINTR) => Cow::Borrowed("EINTR: Interrupted system call"),
        Some(libc::EINVAL) => Cow::Borrowed("EINVAL: Invalid argument"),
        Some(libc::EIO) => Cow::Borrowed("EIO: Input/output error"),
        Some(libc::EISCONN) => Cow::Borrowed("EISCONN: Transport endpoint is already connected"),
        Some(libc::EISDIR) => Cow::Borrowed("EISDIR: Is a directory"),
        Some(libc::EISNAM) => Cow::Borrowed("EISNAM: Is a named type file"),
        Some(libc::EKEYEXPIRED) => Cow::Borrowed("EKEYEXPIRED: Key has expired"),
        Some(libc::EKEYREJECTED) => Cow::Borrowed("EKEYREJECTED: Key was rejected by service"),
        Some(libc::EKEYREVOKED) => Cow::Borrowed("EKEYREVOKED: Key has been revoked"),
        Some(libc::EL2HLT) => Cow::Borrowed("EL2HLT: Level 2 halted"),
        Some(libc::EL2NSYNC) => Cow::Borrowed("EL2NSYNC: Level 2 not synchronized"),
        Some(libc::EL3HLT) => Cow::Borrowed("EL3HLT: Level 3 halted"),
        Some(libc::EL3RST) => Cow::Borrowed("EL3RST: Level 3 reset"),
        Some(libc::ELIBACC) => Cow::Borrowed("ELIBACC: Can not access a needed shared library"),
        Some(libc::ELIBBAD) => Cow::Borrowed("ELIBBAD: Accessing a corrupted shared library"),
        Some(libc::ELIBEXEC) => Cow::Borrowed("ELIBEXEC: Cannot exec a shared library directly"),
        Some(libc::ELIBMAX) => {
            Cow::Borrowed("ELIBMAX: Attempting to link in too many shared libraries")
        }
        Some(libc::ELIBSCN) => Cow::Borrowed("ELIBSCN: .lib section in a.out corrupted"),
        Some(libc::ELNRNG) => Cow::Borrowed("ELNRNG: Link number out of range"),
        Some(libc::ELOOP) => Cow::Borrowed("ELOOP: Too many levels of symbolic links"),
        Some(libc::EMEDIUMTYPE) => Cow::Borrowed("EMEDIUMTYPE: Wrong medium type"),
        Some(libc::EMFILE) => Cow::Borrowed("EMFILE: Too many open files"),
        Some(libc::EMLINK) => Cow::Borrowed("EMLINK: Too many links"),
        Some(libc::EMSGSIZE) => Cow::Borrowed("EMSGSIZE: Message too long"),
        Some(libc::EMULTIHOP) => Cow::Borrowed("EMULTIHOP: Multihop attempted"),
        Some(libc::ENAMETOOLONG) => Cow::Borrowed("ENAMETOOLONG: File name too long"),
        Some(libc::ENAVAIL) => Cow::Borrowed("ENAVAIL: No XENIX semaphores available"),
        Some(libc::ENETDOWN) => Cow::Borrowed("ENETDOWN: Network is down"),
        Some(libc::ENETRESET) => Cow::Borrowed("ENETRESET: Network dropped connection on reset"),
        Some(libc::ENETUNREACH) => Cow::Borrowed("ENETUNREACH: Network is unreachable"),
        Some(libc::ENFILE) => Cow::Borrowed("ENFILE: Too many open files in system"),
        Some(libc::ENOANO) => Cow::Borrowed("ENOANO: No anode"),
        Some(libc::ENOBUFS) => Cow::Borrowed("ENOBUFS: No buffer space available"),
        Some(libc::ENOCSI) => Cow::Borrowed("ENOCSI: No CSI structure available"),
        Some(libc::ENODATA) => Cow::Borrowed("ENODATA: No data available"),
        Some(libc::ENODEV) => Cow::Borrowed("ENODEV: No such device"),
        Some(libc::ENOENT) => Cow::Borrowed("ENOENT: No such file or directory"),
        Some(libc::ENOEXEC) => Cow::Borrowed("ENOEXEC: Exec format error"),
        Some(libc::ENOKEY) => Cow::Borrowed("ENOKEY: Required key not available"),
        Some(libc::ENOLCK) => Cow::Borrowed("ENOLCK: No locks available"),
        Some(libc::ENOLINK) => Cow::Borrowed("ENOLINK: Link has been severed"),
        Some(libc::ENOMEDIUM) => Cow::Borrowed("ENOMEDIUM: No medium found"),
        Some(libc::ENOMEM) => Cow::Borrowed("ENOMEM: Cannot allocate memory"),
        Some(libc::ENOMSG) => Cow::Borrowed("ENOMSG: No message of desired type"),
        Some(libc::ENONET) => Cow::Borrowed("ENONET: Machine is not on the network"),
        Some(libc::ENOPKG) => Cow::Borrowed("ENOPKG: Package not installed"),
        Some(libc::ENOPROTOOPT) => Cow::Borrowed("ENOPROTOOPT: Protocol not available"),
        Some(libc::ENOSPC) => Cow::Borrowed("ENOSPC: No space left on device"),
        Some(libc::ENOSR) => Cow::Borrowed("ENOSR: Out of streams resources"),
        Some(libc::ENOSTR) => Cow::Borrowed("ENOSTR: Device not a stream"),
        Some(libc::ENOSYS) => Cow::Borrowed("ENOSYS: Function not implemented"),
        Some(libc::ENOTBLK) => Cow::Borrowed("ENOTBLK: Block device required"),
        Some(libc::ENOTCONN) => Cow::Borrowed("ENOTCONN: Transport endpoint is not connected"),
        Some(libc::ENOTDIR) => Cow::Borrowed("ENOTDIR: Not a directory"),
        Some(libc::ENOTEMPTY) => Cow::Borrowed("ENOTEMPTY: Directory not empty"),
        Some(libc::ENOTNAM) => Cow::Borrowed("ENOTNAM: Not a XENIX named type file"),
        Some(libc::ENOTRECOVERABLE) => Cow::Borrowed("ENOTRECOVERABLE: State not recoverable"),
        Some(libc::ENOTSOCK) => Cow::Borrowed("ENOTSOCK: Socket operation on non-socket"),
        Some(libc::ENOTSUP) => Cow::Borrowed("ENOTSUP: Not supported"),
        Some(libc::ENOTTY) => Cow::Borrowed("ENOTTY: Inappropriate ioctl for device"),
        Some(libc::ENOTUNIQ) => Cow::Borrowed("ENOTUNIQ: Name not unique on network"),
        Some(libc::ENXIO) => Cow::Borrowed("ENXIO: No such device or address"),
        Some(libc::EOPNOTSUPP) => Cow::Borrowed("EOPNOTSUPP: Operation not supported"),
        Some(libc::EOVERFLOW) => Cow::Borrowed("EOVERFLOW: Value too large for defined data type"),
        Some(libc::EOWNERDEAD) => Cow::Borrowed("EOWNERDEAD: Owner died"),
        Some(libc::EPERM) => Cow::Borrowed("EPERM: Operation not permitted"),
        Some(libc::EPFNOSUPPORT) => Cow::Borrowed("EPFNOSUPPORT: Protocol family not supported"),
        Some(libc::EPIPE) => Cow::Borrowed("EPIPE: Broken pipe"),
        Some(libc::EPROTO) => Cow::Borrowed("EPROTO: Protocol error"),
        Some(libc::EPROTONOSUPPORT) => Cow::Borrowed("EPROTONOSUPPORT: Protocol not supported"),
        Some(libc::EPROTOTYPE) => Cow::Borrowed("EPROTOTYPE: Protocol wrong type for socket"),
        Some(libc::ERANGE) => Cow::Borrowed("ERANGE: Numerical result out of range"),
        Some(libc::EREMCHG) => Cow::Borrowed("EREMCHG: Remote address changed"),
        Some(libc::EREMOTE) => Cow::Borrowed("EREMOTE: Object is remote"),
        Some(libc::EREMOTEIO) => Cow::Borrowed("EREMOTEIO: Remote I/O error"),
        Some(libc::ERESTART) => {
            Cow::Borrowed("ERESTART: Interrupted system call should be restarted")
        }
        Some(libc::ERFKILL) => Cow::Borrowed("ERFKILL: Operation not possible due to RF-kill"),
        Some(libc::EROFS) => Cow::Borrowed("EROFS: Read-only file system"),
        Some(libc::ESHUTDOWN) => {
            Cow::Borrowed("ESHUTDOWN: Cannot send after transport endpoint shutdown")
        }
        Some(libc::ESOCKTNOSUPPORT) => Cow::Borrowed("ESOCKTNOSUPPORT: Socket type not supported"),
        Some(libc::ESPIPE) => Cow::Borrowed("ESPIPE: Illegal seek"),
        Some(libc::ESRCH) => Cow::Borrowed("ESRCH: No such process"),
        Some(libc::ESRMNT) => Cow::Borrowed("ESRMNT: Srmount error"),
        Some(libc::ESTALE) => Cow::Borrowed("ESTALE: Stale file handle"),
        Some(libc::ESTRPIPE) => Cow::Borrowed("ESTRPIPE: Streams pipe error"),
        Some(libc::ETIME) => Cow::Borrowed("ETIME: Timer expired"),
        Some(libc::ETIMEDOUT) => Cow::Borrowed("ETIMEDOUT: Connection timed out"),
        Some(libc::ETOOMANYREFS) => {
            Cow::Borrowed("ETOOMANYREFS: Too many references: cannot splice")
        }
        Some(libc::ETXTBSY) => Cow::Borrowed("ETXTBSY: Text file busy"),
        Some(libc::EUCLEAN) => Cow::Borrowed("EUCLEAN: Structure needs cleaning"),
        Some(libc::EUNATCH) => Cow::Borrowed("EUNATCH: Protocol driver not attached"),
        Some(libc::EUSERS) => Cow::Borrowed("EUSERS: Too many users"),
        Some(libc::EWOULDBLOCK) => Cow::Borrowed("EWOULDBLOCK: Operation would block"),
        Some(libc::EXDEV) => Cow::Borrowed("EXDEV: Invalid cross-device link"),
        Some(libc::EXFULL) => Cow::Borrowed("EXFULL: Exchange full"),
        Some(errno) => Cow::Owned(format_unknown_errno(errno)),
        None => format_non_errno(e),
    };

    if let Some(syscall) = syscall {
        let mut s = result.into_owned();
        s.push_str(" from syscall '");
        s.push_str(syscall);
        s.push('\'');
        Cow::Owned(s)
    } else {
        result
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
    // Have to do a bit of a dance here to ensure it's displayed properly. The tricks I use to
    // reduce copying are pretty non-standard.
    match normalize_errno(e, Some(syscall_name)) {
        Cow::Borrowed(s) => std::panic::panic_any(s),
        Cow::Owned(s) => std::panic::panic_any(s),
    }
}
