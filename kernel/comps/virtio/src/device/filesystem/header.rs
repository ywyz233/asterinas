use ostd::{
    early_print, mm::{VmReader, VmWriter}, Error, Pod
};
use crate::{device::filesystem::fuse::*, queue::VirtQueue};
use alloc::{
    string::ToString, vec::Vec, vec,
};

#[derive(Debug)]
#[repr(C)]
pub struct VirtioFsReq{
    //Device readable
    pub headerin: FuseInHeader,
    pub datain: Vec<u8>,

    //Device writable
    pub headerout: FuseOutHeader,
    pub dataout: Vec<u8>,
}

impl VirtioFsReq{
    pub fn into_bytes(&self) -> Vec<u8>{
        let fuse_in_header = self.headerin.as_bytes();
        let datain = self.datain.as_slice();
        let fuse_out_header = self.headerout.as_bytes();
        let dataout = self.dataout.as_slice();

        
        let total_len = fuse_in_header.len() + datain.len() + fuse_out_header.len() + dataout.len();

        let mut concat_req= vec![0u8; total_len];
        concat_req[0..fuse_in_header.len()].copy_from_slice(fuse_in_header);
        concat_req[fuse_in_header.len()..(fuse_in_header.len() + datain.len())].copy_from_slice(datain);
        
        concat_req
    }

    // pub fn from_bytes(bytes: &[u8]) -> Self{
    //     let mut base_idx = 0;
    //     let headerin_len = size_of::<FuseInHeader>();
    //     let headerin = FuseInHeader::from_bytes(&bytes[base_idx..base_idx + headerin_len]);
    //     base_idx += headerin_len;
    //     let data_in_len = headerin.len as usize - headerin_len;
    //     let         
    // }
}

/// Define the Fuse Device function interface
pub trait AnyFuseDevice{
    // Util functions
    fn send(&self, concat_req: &[u8], request_queue_idx: usize, locked_request_queue: &mut VirtQueue, readable_len: usize, writeable_start: usize);
    fn sendhp(&self, concat_req: &[u8], locked_hp_queue: &mut VirtQueue, readable_len: usize, writeable_start: usize);
    // Functions defined in Fuse
    fn init(&self);
    fn handle_init(&self, init_out: FuseInitOut) -> bool;
    fn readdir(&self, nodeid: u64, fh: u64, offset: u64, size: u32);
    fn opendir(&self, nodeid: u64, flags: u32);
    fn mkdir(&self, nodeid: u64, mode: u32, mask: u32, name: &str);
    fn lookup(&self, nodeid: u64, name: &str);
    fn open(&self, nodeid: u64, flags: u32);
    fn read(&self, nodeid: u64, fh: u64, offset: u64, size: u32);
    fn write(&self, nodeid: u64, fh: u64, offset: u64, data: &str);
    fn mknod(&self, nodeid: u64, mode: u32, mask: u32, name: &str);
    fn rename(&self, nodeid: u64, newdir: u64, oldname: &str, newname: &str);
    fn rename2(&self, nodeid: u64, newdir: u64, flags: u32, oldname: &str, newname: &str);
    fn forget(&self, nodeid: u64, nlookup: u64);
    fn getattr(&self, nodeid: u64, flags: u32, fh: u64);
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
    fn readlink(&self, nodeid: u64, out_buf_size: u32);
    /// name: the name of symbolic link file, 
    /// link_name: the name of target
    fn symlink(&self, nodeid: u64, name: &str, link_name: &str);
    fn rmdir(&self, nodeid: u64, name: &str);
    fn unlink(&self, nodeid: u64, name: &str);
    fn link(&self, nodeid: u64, oldnodeid: u64, name: &str);
    fn statfs(&self, nodeid: u64);
    fn copyfilerange(&self, nodeid: u64, fh_in: u64, off_in: u64, nodeid_out: u64, fh_out: u64, off_out: u64, len: u64, flags: u64);
    fn release(&self, nodeid: u64, fh: u64, flags: u32, release_flags: u32, lock_owner: u64);
    fn releasedir(&self, nodeid: u64, fh: u64, flags: u32);
    fn fsync(&self, nodeid: u64, fh: u64, fsync_flags: u32);
    
    /// setxattr_flags is set as zero
    fn setxattr(&self, nodeid: u64, name: &str, value: &[u8], flags: u32);
    /// The needed size of buffer will be returned if out_buf_size = 0
    fn getxattr(&self, nodeid: u64, name: &str, out_buf_size: u32);
    /// The needed size of buffer will be returned if out_buf_size = 0
    fn listxattr(&self, nodeid: u64, out_buf_size: u32);
    fn removexattr(&self, nodeid: u64, name: &str);
    fn access(&self, nodeid: u64, mask: u32);

    fn interrupt(&self);
    fn batchforget(&self, forget_list: &[(u64, u64)]);

    fn flush(&self, nodeid: u64, fh: u64, lock_owner: u64);
    fn fallocate(&self, nodeid: u64, fh: u64, offset: u64, len: u64, mode: u32);
    fn lseek(&self, nodeid: u64, fh: u64, offset: u64, whence: u32);
}

///FuseDirent with the file name
pub struct FuseDirentWithName{
    pub dirent: FuseDirent,
    pub name: Vec<u8>,
}

///Contain all directory entries for one directory
pub struct FuseReaddirOut{
    pub dirents: Vec<FuseDirentWithName>,
}

impl FuseReaddirOut{
    /// Read all directory entries from the buffer
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
            // early_print!("len: {:?} ,dirlen: {:?}, name_len: {:?}\n", len, size_of::<FuseDirent>() as u32 + dirent.namelen, dirent.namelen);
            len -= size_of::<FuseDirent>() as i32 + dirent.namelen as i32 + pad_len as i32;
        }
        FuseReaddirOut { dirents: dirents }
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