use ostd::{Pod, Error};
use crate::device::filesystem::fuse::*;
use alloc::{
    vec, 
    vec::Vec
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

pub trait AnyFuseDevice{
    // Send Init Request to Device.
    fn init(&self);
    fn readdir(&self, nodeid: u64, fh: u64, offset: u64, size: u32);
    fn opendir(&self, nodeid: u64, flags: u32);
}

pub fn to_opcode(val: u32) -> Result<FuseOpcode, Error>{
    match(val) {
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