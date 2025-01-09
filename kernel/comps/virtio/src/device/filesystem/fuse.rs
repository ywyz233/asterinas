use bitflags;
use ostd::Pod;

/// Version number of this interface.
pub const FUSE_KERNEL_VERSION: u32 = 7;

/// Minor version number of this interface.
pub const FUSE_KERNEL_MINOR_VERSION: u32 = 38;

/// Minimum Minor version number supported. If client sends a minor
/// number lesser than this, we don't support it.
pub const FUSE_MIN_KERNEL_MINOR_VERSION: u32 = 27;


// Flags in Init Message
const FUSE_ASYNC_READ: u64			= 1 << 0;
const FUSE_POSIX_LOCKS: u64			= 1 << 1;
const FUSE_FILE_OPS: u64			= 1 << 2;
const FUSE_ATOMIC_O_TRUNC: u64		= 1 << 3;
const FUSE_EXPORT_SUPPORT: u64		= 1 << 4;
const FUSE_BIG_WRITES: u64			= 1 << 5;
const FUSE_DONT_MASK: u64			= 1 << 6;
const FUSE_SPLICE_WRITE: u64		= 1 << 7;
const FUSE_SPLICE_MOVE: u64			= 1 << 8;
const FUSE_SPLICE_READ: u64			= 1 << 9;
const FUSE_FLOCK_LOCKS: u64			= 1 << 10;
const FUSE_HAS_IOCTL_DIR: u64		= 1 << 11;
const FUSE_AUTO_INVAL_DATA: u64		= 1 << 12;
const FUSE_DO_READDIRPLUS: u64		= 1 << 13;
const FUSE_READDIRPLUS_AUTO: u64	= 1 << 14;
const FUSE_ASYNC_DIO: u64			= 1 << 15;
const FUSE_WRITEBACK_CACHE: u64		= 1 << 16;
const FUSE_NO_OPEN_SUPPORT: u64		= 1 << 17;
const FUSE_PARALLEL_DIROPS: u64  	= 1 << 18;
const FUSE_HANDLE_KILLPRIV: u64		= 1 << 19;
const FUSE_POSIX_ACL: u64			= 1 << 20;
const FUSE_ABORT_ERROR: u64			= 1 << 21;
const FUSE_MAX_PAGES: u64			= 1 << 22;
const FUSE_CACHE_SYMLINKS: u64		= 1 << 23;
const FUSE_NO_OPENDIR_SUPPORT: u64	= 1 << 24;
const FUSE_EXPLICIT_INVAL_DATA: u64	= 1 << 25;
const FUSE_MAP_ALIGNMENT: u64		= 1 << 26;
const FUSE_SUBMOUNTS: u64			= 1 << 27;
const FUSE_HANDLE_KILLPRIV_V2: u64	= 1 << 28;
const FUSE_SETXATTR_EXT: u64		= 1 << 29;
const FUSE_INIT_EXT: u64			= 1 << 30;
const FUSE_INIT_RESERVED: u64		= 1 << 31;
/* bits 32..63 get shifted down 32 bits into the flags2 field */
const FUSE_SECURITY_CTX: u64		= 1u64 << 32;
const FUSE_HAS_INODE_DAX: u64		= 1u64 << 33;

bitflags::bitflags! {
	pub struct FuseInitFlags: u64 {
		const FUSE_INIT_EXT = FUSE_INIT_EXT;
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

