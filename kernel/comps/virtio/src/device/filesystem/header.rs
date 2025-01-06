use ostd::Pod;
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