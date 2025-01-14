use ostd::{
    mm::{VmReader, VmWriter}, Error
};
use crate::{device::filesystem::fuse::*, queue::VirtQueue};
use alloc::{
    vec::Vec, vec,
};

/// Define the Fuse Device function interface
pub trait AnyFuseDevice{
    // Util functions
    fn send(&self, concat_req: &[u8], request_queue_idx: usize, locked_request_queue: &mut VirtQueue, readable_len: usize, writeable_start: usize);
    fn sendhp(&self, concat_req: &[u8], locked_hp_queue: &mut VirtQueue, readable_len: usize, writeable_start: usize);
    
    // Functions defined in Fuse

    /// When the device is created, init should be called to negotiate the FUSE
    /// protocol version, request structure, endianess and etc. with virtiofs daemon.
    fn init(&self);
    
    /// Handler of received init from virtiofs daemon. 
    fn handle_init(&self, init_out: FuseInitOut) -> bool;
    
    /// Read contents in a directory.
    fn readdir(&self, nodeid: u64, fh: u64, offset: u64, size: u32);
    
    /// Open a directory, returns file handler fh.
    fn opendir(&self, nodeid: u64, flags: u32);
    
    /// Make a directory under the given parent directory.
    fn mkdir(&self, nodeid: u64, mode: u32, umask: u32, name: &str);
    
    /// Lookup the inode index and attributes of a file through file name.
    fn lookup(&self, nodeid: u64, name: &str);
    
    /// Open a file, returns a file handler fh.
    fn open(&self, nodeid: u64, flags: u32);
    
    /// Read a file from offset bytes to offset + size bytes.
    fn read(&self, nodeid: u64, fh: u64, offset: u64, size: u32);
    
    /// Write data to the file at offset position.
    fn write(&self, nodeid: u64, fh: u64, offset: u64, data: &[u8]);
    
    /// Make an inode(file) with the given privilege and file name.
    fn mknod(&self, nodeid: u64, mode: u32, mask: u32, name: &str);
    
    /// Rename a file/directory.
    fn rename(&self, nodeid: u64, newdir: u64, oldname: &str, newname: &str);
    
    /// Support advanced operation based on rename operation. Advanced operation is
    /// specified by flags
    fn rename2(&self, nodeid: u64, newdir: u64, flags: u32, oldname: &str, newname: &str);
    
    /// High priority operation, tell the device it should release the given file/directory.
    fn forget(&self, nodeid: u64, nlookup: u64);
    
    /// Get the attribute of specified file.
    fn getattr(&self, nodeid: u64, flags: u32, fh: u64);
    
    /// Set the attribute of specified file.
    fn setattr(
        &self, 
        nodeid: u64, 
        valid: u32, 
        fh: u64, 
        size: u64, 
        lock_owner: u64, 
        atime: u64, 
        mtime: u64, 
        ctime: u64, 
        atimensec: u32, 
        mtimensec: u32, 
        ctimensec: u32,
        mode: u32,
        uid: u32,
        gid: u32,
    );

    /// Read the content of symbolic link.
    fn readlink(&self, nodeid: u64, out_buf_size: u32);

    /// Create a symbolic link.
    /// name: the name of symbolic link file, 
    /// link_name: the name of target
    fn symlink(&self, nodeid: u64, name: &str, link_name: &str);
    
    /// Remove the specified directory.
    fn rmdir(&self, nodeid: u64, name: &str);
    
    /// Delete a hard link, when the link counter is reduced to zero, the file will be deleted.
    fn unlink(&self, nodeid: u64, name: &str);
    
    /// Create a hard link to the specified file.
    fn link(&self, nodeid: u64, oldnodeid: u64, name: &str);
    
    /// Get the status of the file system.
    fn statfs(&self, nodeid: u64);
    
    /// Copy a part of the content of the source file to destination file.
    fn copyfilerange(&self, nodeid: u64, fh_in: u64, off_in: u64, nodeid_out: u64, fh_out: u64, off_out: u64, len: u64, flags: u64);
    
    /// Release the file handler of a file.
    fn release(&self, nodeid: u64, fh: u64, flags: u32, release_flags: u32, lock_owner: u64);
    
    /// Release the file handler of a directory.
    fn releasedir(&self, nodeid: u64, fh: u64, flags: u32);
    
    /// Synchronize the data and metadata of a file to disk.
    fn fsync(&self, nodeid: u64, fh: u64, fsync_flags: u32);
    
    /// Synchronize the data and metadata of a directory to disk.
    fn fsyncdir(&self, nodeid: u64, fh: u64, fsync_flags: u32);
    
    /// Set extra attribute for a file.
    /// setxattr_flags is set as zero
    fn setxattr(&self, nodeid: u64, name: &str, value: &[u8], flags: u32);
    
    /// Get the value of an extra attribute.
    /// The needed size of buffer will be returned if out_buf_size = 0
    fn getxattr(&self, nodeid: u64, name: &str, out_buf_size: u32);
    
    /// List all keys of extra attrbute of a file.
    /// The needed size of buffer will be returned if out_buf_size = 0
    fn listxattr(&self, nodeid: u64, out_buf_size: u32);
    
    /// Remove an extra attribute of a file.
    fn removexattr(&self, nodeid: u64, name: &str);
    
    /// Check the privilege of a user on specified file or directory.
    fn access(&self, nodeid: u64, mask: u32);

    /// High priority request, tell the device to cancel the processing request.
    fn interrupt(&self);
    
    /// High priority request, equivalent to multiple call to forget function.
    fn batchforget(&self, forget_list: &[(u64, u64)]);

    /// Write the data and metadata of a file to underlying data structure.
    fn flush(&self, nodeid: u64, fh: u64, lock_owner: u64);
    
    /// Pre-reserve space in the specified file.
    fn fallocate(&self, nodeid: u64, fh: u64, offset: u64, len: u64, mode: u32);
    
    /// Set the position of file cursor.
    fn lseek(&self, nodeid: u64, fh: u64, offset: u64, whence: u32);
    
    /// Create a file and returns with a file handler fh. 
    fn create(&self, nodeid: u64, flags: u32, mode: u32, mask: u32, name: &str);
    
    /// Supports advanced operation based on readdir operation. 
    fn readdirplus(&self, nodeid: u64, fh: u64, offset: u64, size: u32);
}

///FuseDirent with the file name
pub struct FuseDirentWithName{
    pub dirent: FuseDirent,
    pub name: Vec<u8>,
}

pub struct FuseDirentWithNamePlus{
    pub dirent: FuseDirent,
    pub name: Vec<u8>,
    pub entry: FuseEntryOut,
}

///Contain all directory entries for one directory
pub struct FuseReaddirOut{
    pub dirents: Vec<FuseDirentWithName>,
}

pub struct FuseReaddirplusOut{
    pub dirents: Vec<FuseDirentWithNamePlus>,
}

impl FuseReaddirOut{
    /// Helper function, read all directory entries from the buffer
    pub fn read_dirent(reader: &mut VmReader<'_, ostd::mm::Infallible>, out_header: FuseOutHeader) -> FuseReaddirOut{
        let mut len  = out_header.len as i32 - size_of::<FuseOutHeader>() as i32;
        let mut dirents: Vec<FuseDirentWithName> = Vec::new();

        // For paddings between dirents
        let mut padding: Vec<u8> = vec![0 as u8; 8];
        while len > 0{
            let dirent = reader.read_val::<FuseDirent>().unwrap();
            let mut file_name: Vec<u8>;
            
            file_name = vec![0 as u8; dirent.namelen as usize];
            let mut writer = VmWriter::from(file_name.as_mut_slice());
            writer.write(reader);

            let pad_len = (8 - (dirent.namelen & 0x7)) & 0x7; // pad to multiple of 8 bytes
            let mut pad_writer = VmWriter::from(&mut padding[0..pad_len as usize]);
            pad_writer.write(reader);


            dirents.push(FuseDirentWithName{
                dirent: dirent,
                name: file_name,
            });
            len -= size_of::<FuseDirent>() as i32 + dirent.namelen as i32 + pad_len as i32;
        }
        FuseReaddirOut { dirents: dirents }
    }
}

impl FuseReaddirplusOut{
    /// Helper function, read all directory entries from the buffer
    pub fn read_dirent(reader: &mut VmReader<'_, ostd::mm::Infallible>, out_header: FuseOutHeader) -> FuseReaddirplusOut{
        let mut len  = out_header.len as i32 - size_of::<FuseOutHeader>() as i32;
        let mut dirents: Vec<FuseDirentWithNamePlus> = Vec::new();

        // For paddings between dirents
        let mut padding: Vec<u8> = vec![0 as u8; 8];
        while len > 0{
            let entry = reader.read_val::<FuseEntryOut>().unwrap();
            let dirent = reader.read_val::<FuseDirent>().unwrap();
            let mut file_name: Vec<u8>;
            
            file_name = vec![0 as u8; dirent.namelen as usize];
            let mut writer = VmWriter::from(file_name.as_mut_slice());
            writer.write(reader);

            let pad_len = (8 - (dirent.namelen & 0x7)) & 0x7; // pad to multiple of 8 bytes
            let mut pad_writer = VmWriter::from(&mut padding[0..pad_len as usize]);
            pad_writer.write(reader);


            dirents.push(FuseDirentWithNamePlus{
                entry: entry,
                dirent: dirent,
                name: file_name,
            });
            len -= size_of::<FuseEntryOut>() as i32 + size_of::<FuseDirent>() as i32 + dirent.namelen as i32 + pad_len as i32;
        }
        FuseReaddirplusOut { dirents: dirents }
    }
}


/// Pad the file name/path name to multiple of 8 bytes with '\0'
/// If repr_c is set, then one additional '\0' will be added at the end of name as if it is originally in name.
pub fn fuse_pad_str(name: &str, repr_c: bool)-> Vec<u8>{
    let name_len = name.len() as u32 + if repr_c {1} else {0};
    let name_pad_len = name_len  + ((8 - (name_len & 0x7)) & 0x7); //Pad to multiple of 8 bytes 
    let mut prepared_name: Vec<u8> = name.as_bytes().to_vec();
    prepared_name.resize(name_pad_len as usize, 0);
    prepared_name
}

/// Map the opcode value to corresponding enum type
pub fn as_opcode(val: u32) -> Result<FuseOpcode, Error>{
    match val {
        1 => Ok(FuseOpcode::FuseLookup),
        2 => Ok(FuseOpcode::FuseForget),
        3 => Ok(FuseOpcode::FuseGetattr),
        4 => Ok(FuseOpcode::FuseSetattr),
        5 => Ok(FuseOpcode::FuseReadlink),
        6 => Ok(FuseOpcode::FuseSymlink),
        8 => Ok(FuseOpcode::FuseMknod),
        9 => Ok(FuseOpcode::FuseMkdir),
        10 => Ok(FuseOpcode::FuseUnlink),
        11 => Ok(FuseOpcode::FuseRmdir),
        12 => Ok(FuseOpcode::FuseRename),
        13 => Ok(FuseOpcode::FuseLink),
        14 => Ok(FuseOpcode::FuseOpen),
        15 => Ok(FuseOpcode::FuseRead),
        16 => Ok(FuseOpcode::FuseWrite),
        17 => Ok(FuseOpcode::FuseStatfs),
        18 => Ok(FuseOpcode::FuseRelease),
        20 => Ok(FuseOpcode::FuseFsync),
        21 => Ok(FuseOpcode::FuseSetxattr),
        22 => Ok(FuseOpcode::FuseGetxattr),
        23 => Ok(FuseOpcode::FuseListxattr),
        24 => Ok(FuseOpcode::FuseRemovexattr),
        25 => Ok(FuseOpcode::FuseFlush),
        26 => Ok(FuseOpcode::FuseInit),
        27 => Ok(FuseOpcode::FuseOpendir),
        28 => Ok(FuseOpcode::FuseReaddir),
        29 => Ok(FuseOpcode::FuseReleasedir),
        30 => Ok(FuseOpcode::FuseFsyncdir),
        31 => Ok(FuseOpcode::FuseGetlk),
        32 => Ok(FuseOpcode::FuseSetlk),
        33 => Ok(FuseOpcode::FuseSetlkw),
        34 => Ok(FuseOpcode::FuseAccess),
        35 => Ok(FuseOpcode::FuseCreate),
        36 => Ok(FuseOpcode::FuseInterrupt),
        37 => Ok(FuseOpcode::FuseBmap),
        38 => Ok(FuseOpcode::FuseDestroy),
        39 => Ok(FuseOpcode::FuseIoctl),
        40 => Ok(FuseOpcode::FusePoll),
        41 => Ok(FuseOpcode::FuseNotifyReply),
        42 => Ok(FuseOpcode::FuseBatchForget),
        43 => Ok(FuseOpcode::FuseFallocate),
        44 => Ok(FuseOpcode::FuseReaddirplus),
        45 => Ok(FuseOpcode::FuseRename2),
        46 => Ok(FuseOpcode::FuseLseek),
        47 => Ok(FuseOpcode::FuseCopyFileRange),
        48 => Ok(FuseOpcode::FuseSetupmapping),
        49 => Ok(FuseOpcode::FuseRemovemapping),
        50 => Ok(FuseOpcode::FuseSyncfs),
        51 => Ok(FuseOpcode::FuseTmpfile),
        52 => Ok(FuseOpcode::FuseStatx),
        4096 => Ok(FuseOpcode::CuseInit),
        1048576 => Ok(FuseOpcode::CuseInitBswapReserved),
        436207616 => Ok(FuseOpcode::FuseInitBswapReserved),
        _ => Err(Error::InvalidArgs),
    }
}