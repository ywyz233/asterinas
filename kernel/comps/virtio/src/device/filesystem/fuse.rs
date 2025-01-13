use bitflags;
use ostd::Pod;

/// Version number of this interface.
pub const FUSE_KERNEL_VERSION: u32 = 7;

/// Minor version number of this interface.
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 38;

/// Minimum Minor version number supported. If client sends a minor
/// number lesser than this, we don't support it.
pub const FUSE_MIN_KERNEL_MINOR_VERSION: u32 = 38;


// Flags in Init Message
pub const FUSE_ASYNC_READ: u64			= 1 << 0;
pub const FUSE_POSIX_LOCKS: u64			= 1 << 1;
pub const FUSE_FILE_OPS: u64			= 1 << 2;
pub const FUSE_ATOMIC_O_TRUNC: u64		= 1 << 3;
pub const FUSE_EXPORT_SUPPORT: u64		= 1 << 4;
pub const FUSE_BIG_WRITES: u64			= 1 << 5;
pub const FUSE_DONT_MASK: u64			= 1 << 6;
pub const FUSE_SPLICE_WRITE: u64		= 1 << 7;
pub const FUSE_SPLICE_MOVE: u64			= 1 << 8;
pub const FUSE_SPLICE_READ: u64			= 1 << 9;
pub const FUSE_FLOCK_LOCKS: u64			= 1 << 10;
pub const FUSE_HAS_IOCTL_DIR: u64		= 1 << 11;
pub const FUSE_AUTO_INVAL_DATA: u64		= 1 << 12;
pub const FUSE_DO_READDIRPLUS: u64		= 1 << 13;
pub const FUSE_READDIRPLUS_AUTO: u64	= 1 << 14;
pub const FUSE_ASYNC_DIO: u64			= 1 << 15;
pub const FUSE_WRITEBACK_CACHE: u64		= 1 << 16;
pub const FUSE_NO_OPEN_SUPPORT: u64		= 1 << 17;
pub const FUSE_PARALLEL_DIROPS: u64  	= 1 << 18;
pub const FUSE_HANDLE_KILLPRIV: u64		= 1 << 19;
pub const FUSE_POSIX_ACL: u64			= 1 << 20;
pub const FUSE_ABORT_ERROR: u64			= 1 << 21;
pub const FUSE_MAX_PAGES: u64			= 1 << 22;
pub const FUSE_CACHE_SYMLINKS: u64		= 1 << 23;
pub const FUSE_NO_OPENDIR_SUPPORT: u64	= 1 << 24;
pub const FUSE_EXPLICIT_INVAL_DATA: u64	= 1 << 25;
pub const FUSE_MAP_ALIGNMENT: u64		= 1 << 26;
pub const FUSE_SUBMOUNTS: u64			= 1 << 27;
pub const FUSE_HANDLE_KILLPRIV_V2: u64	= 1 << 28;
pub const FUSE_SETXATTR_EXT: u64		= 1 << 29;
pub const FUSE_INIT_EXT: u64			= 1 << 30;
pub const FUSE_INIT_RESERVED: u64		= 1 << 31;
/* bits 32..63 get shifted down 32 bits into the flags2 field */
pub const FUSE_SECURITY_CTX: u64			= 1u64 << 32;
pub const FUSE_HAS_INODE_DAX: u64			= 1u64 << 33;
pub const FUSE_CREATE_SUPP_GROUP: u64		= 1u64 << 34;
pub const FUSE_HAS_EXPIRE_ONLY: u64			= 1u64 << 35;
pub const FUSE_DIRECT_IO_ALLOW_MMAP: u64	= 1u64 << 36;
pub const FUSE_PASSTHROUGH: u64				= 1u64 << 37;
pub const FUSE_NO_EXPORT_SUPPORT: u64		= 1u64 << 38;
pub const FUSE_HAS_RESEND: u64				= 1u64 << 39;

// Getattr flags.
pub const FUSE_GETATTR_FH: u32 = 1 << 0;

/// Delayed write from page cache, file handle is guessed.
pub const FUSE_WRITE_CACHE: u32 = 1 << 0;

/// `lock_owner` field is valid.
pub const FUSE_WRITE_LOCKOWNER: u32 = 1 << 1;

/// Kill suid and sgid bits
pub const FUSE_WRITE_KILL_PRIV: u32 = 1 << 2;


bitflags::bitflags! {
	pub struct FuseInitFlags: u64 {
		const FUSE_INIT_EXT = FUSE_INIT_EXT;
		const FUSE_SETXATTR_EXT = FUSE_SETXATTR_EXT;
	}
}

// Bitmasks for `fuse_setattr_in.valid`.
const FATTR_MODE: u32 = 1 << 0;
const FATTR_UID: u32 = 1 << 1;
const FATTR_GID: u32 = 1 << 2;
const FATTR_SIZE: u32 = 1 << 3;
const FATTR_ATIME: u32 = 1 << 4;
const FATTR_MTIME: u32 = 1 << 5;
pub const FATTR_FH: u32 = 1 << 6;
const FATTR_ATIME_NOW: u32 = 1 << 7;
const FATTR_MTIME_NOW: u32 = 1 << 8;
pub const FATTR_LOCKOWNER: u32 = 1 << 9;
const FATTR_CTIME: u32 = 1 << 10;
const FATTR_KILL_SUIDGID: u32 = 1 << 11;

bitflags::bitflags! {
    pub struct FuseSetattrValid: u32 {
        const MODE = FATTR_MODE;
        const UID = FATTR_UID;
        const GID = FATTR_GID;
        const SIZE = FATTR_SIZE;
        const ATIME = FATTR_ATIME;
        const MTIME = FATTR_MTIME;
        const ATIME_NOW = FATTR_ATIME_NOW;
        const MTIME_NOW = FATTR_MTIME_NOW;
        const CTIME = FATTR_CTIME;
        const KILL_SUIDGID = FATTR_KILL_SUIDGID;
    }
}

// setxattr flags
/// Clear SGID when system.posix_acl_access is set
const FUSE_SETXATTR_ACL_KILL_SGID: u32 = 1 << 0;

bitflags::bitflags! {
    pub struct FuseSetxattrFlags: u32 {
        /// Clear SGID when system.posix_acl_access is set
        const SETXATTR_ACL_KILL_SGID = FUSE_SETXATTR_ACL_KILL_SGID;
    }
}

pub enum FuseOpcode{
    FuseLookup	            = 1,
	FuseForget		        = 2,  /* no reply */
	FuseGetattr		        = 3,
	FuseSetattr		        = 4,
    FuseReadlink		    = 5,
	FuseSymlink		        = 6,
	FuseMknod		        = 8,
	FuseMkdir		        = 9,
	FuseUnlink		        = 10,
	FuseRmdir		        = 11,
	FuseRename		        = 12,
	FuseLink		        = 13,
	FuseOpen		        = 14,   
	FuseRead		        = 15,
	FuseWrite		        = 16,
	FuseStatfs		        = 17,
	FuseRelease		        = 18,
	FuseFsync		        = 20,
	FuseSetxattr	        = 21,
	FuseGetxattr	        = 22,
	FuseListxattr	        = 23,
	FuseRemovexattr	        = 24,
	FuseFlush		        = 25,
	FuseInit		        = 26,
	FuseOpendir		        = 27,
	FuseReaddir		        = 28,
	FuseReleasedir	        = 29,
	FuseFsyncdir	        = 30,
	FuseGetlk		        = 31,
	FuseSetlk		        = 32,
	FuseSetlkw		        = 33,
	FuseAccess		        = 34,
	FuseCreate		        = 35,
	FuseInterrupt	        = 36,
	FuseBmap		        = 37,
	FuseDestroy		        = 38,
	FuseIoctl		        = 39,
	FusePoll		        = 40,
	FuseNotifyReply	        = 41,
	FuseBatchForget	        = 42,
	FuseFallocate	        = 43,
	FuseReaddirplus	        = 44,
	FuseRename2		        = 45,
	FuseLseek		        = 46,
	FuseCopyFileRange	    = 47,
	FuseSetupmapping	    = 48,
	FuseRemovemapping	    = 49,
	FuseSyncfs		        = 50,
	FuseTmpfile		        = 51,
	FuseStatx		        = 52,

	/* CUSE specific operations */
	CuseInit		        = 4096,

	/* Reserved opcodes: helpful to detect structure endian-ness */
	CuseInitBswapReserved	= 1048576,	/* CUSE_INIT << 8 */
	FuseInitBswapReserved	= 436207616,	/* FUSE_INIT << 24 */
}

pub enum FuseNotifyCode {
	FuseNotifyPoll   = 1,
	FuseNotifyInvalInode = 2,
	FuseNotifyInvalEntry = 3,
	FuseNotifyStore = 4,
	FuseNotifyRetrieve = 5,
	FuseNotifyDelete = 6,
	FuseNotifyResend = 7,
	FuseNotifyCodeMax,
}

#[derive(Debug, Pod, Clone, Copy)]
#[repr(C)]
pub struct FuseInHeader {
	pub len: u32,
	pub opcode: u32,
	pub unique: u64,
	pub nodeid: u64,
	pub uid: u32,
	pub gid: u32,
	pub pid: u32,
	pub total_extlen: u16, /* length of extensions in 8byte units */
	pub padding: u16,
}

#[derive(Debug, Pod, Clone, Copy)]
#[repr(C)]
pub struct FuseOutHeader {
    pub len: u32,
    pub error: i32,
    pub unique: u64,
}

#[derive(Debug, Pod, Clone, Copy)]
#[repr(C)]
pub struct FuseInitIn {
	pub major: u32,
	pub minor: u32,
	pub max_readahead: u32,
	pub flags: u32,
	pub flags2: u32,
	pub unused: [u32; 11],
}

// #define FUSE_COMPAT_INIT_OUT_SIZE 8
// #define FUSE_COMPAT_22_INIT_OUT_SIZE 24

#[derive(Debug, Pod, Clone, Copy)]
#[repr(C)]
pub struct FuseInitOut {
	pub major: u32,
	pub minor: u32,
	pub max_readahead: u32,
	pub flags: u32,
	pub max_background: u16,
	pub congestion_threshold: u16,
	pub max_write: u32,
	pub time_gran: u32,
	pub max_pages: u16,
	pub map_alignment: u16,
	pub flags2: u32,
	pub max_stack_depth: u32,
	pub unused: [u32; 6],
}

#[derive(Debug, Pod, Clone, Copy)]
#[repr(C)]
pub struct FuseReadIn {
    pub fh: u64,
    pub offset: u64,
    pub size: u32,
    pub read_flags: u32,
    pub lock_owner: u64,
    pub flags: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseWriteIn {
    pub fh: u64,
    pub offset: u64,
    pub size: u32,
    pub write_flags: u32,
    pub lock_owner: u64,
    pub flags: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseWriteOut {
    pub size: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseDirent {
    pub ino: u64,
    pub off: u64,
    pub namelen: u32,
    pub type_: u32,
    // char name[];
}

#[derive(Debug, Pod, Clone, Copy)]
#[repr(C)]
pub struct FuseOpenIn {
    pub flags: u32,
    pub open_flags: u32,
}

#[derive(Debug, Pod, Clone, Copy)]
#[repr(C)]
pub struct FuseOpenOut {
    pub fh: u64,
    pub open_flags: u32,
    pub backing_id: u32,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseAttr {
    pub ino: u64,
    pub size: u64,
    pub blocks: u64,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
    pub atimensec: u32,
    pub mtimensec: u32,
    pub ctimensec: u32,
    pub mode: u32,
    pub nlink: u32,
    pub uid: u32,
    pub gid: u32,
    pub rdev: u32,
    pub blksize: u32,
    pub flags: u32,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseEntryOut {
    pub nodeid: u64,      /* Inode ID */
    pub generation: u64,  /* Inode generation: nodeid:gen must be unique for the fs's lifetime */
    pub entry_valid: u64, /* Cache timeout for the name */
    pub attr_valid: u64,  /* Cache timeout for the attributes */
    pub entry_valid_nsec: u32,
    pub attr_valid_nsec: u32,
    pub attr: FuseAttr,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseMkdirIn {
    pub mode: u32, // octal mode
    pub umask: u32, 
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseMknodIn {
    pub mode: u32, // octal mode
	pub rdev: u32,
    pub umask: u32,
	pub padding: u32, 
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseGetattrIn {
    pub flags: u32,
    pub dummy: u32,
    pub fh: u64,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseRenameIn {
	pub newdir: u64,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseAttrOut {
    pub attr_valid: u64, /* Cache timeout for the attributes */
    pub attr_valid_nsec: u32,
    pub dummy: u32,
    pub attr: FuseAttr,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseSetattrIn {
    pub valid: u32,
    pub padding: u32,
    pub fh: u64,
    pub size: u64,
    pub lock_owner: u64,
    pub atime: u64,
    pub mtime: u64,
    pub ctime: u64,
    pub atimensec: u32,
    pub mtimensec: u32,
    pub ctimensec: u32,
    pub mode: u32,
    pub unused4: u32,
    pub uid: u32,
    pub gid: u32,
    pub unused5: u32,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseRename2In {
	pub newdir: u64,
	pub flags: u32,
	pub padding: u32,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseForgetIn {
    pub nlookup: u64,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseLinkIn {
    pub oldnodeid: u64,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseKstatfs {
    pub blocks: u64,
    pub bfree: u64,
    pub bavail: u64,
    pub files: u64,
    pub ffree: u64,
    pub bsize: u32,
    pub namelen: u32,
    pub frsize: u32,
    pub padding: u32,
    pub spare: [u32; 6],
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseCopyfilerangeIn {
    pub fh_in: u64,
    pub off_in: u64,
    pub nodeid_out: u64,
    pub fh_out: u64,
    pub off_out: u64,
    pub len: u64,
    pub flags: u64,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseSetxattrIn {
    pub size: u32,
    pub flags: u32,
    pub setxattr_flags: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseSetxattrInCompat {
    pub size: u32,
    pub flags: u32,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseGetxattrIn {
    pub size: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseGetxattrOut {
    pub size: u32,
    pub padding: u32,
}

#[repr(C)]
#[derive(Debug, Pod, Copy, Clone)]
pub struct FuseAccessIn {
    pub mask: u32,
    pub padding: u32,
}