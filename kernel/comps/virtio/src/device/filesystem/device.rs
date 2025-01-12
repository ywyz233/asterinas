use core::sync::atomic::{
    AtomicU64,
    Ordering,
};

use ostd::{
    early_print, 
    mm::{DmaDirection, DmaStream, DmaStreamSlice, FrameAllocOptions, VmReader, VmWriter}, 
    sync::{SpinLock, RwLock},
    trap::TrapFrame, 
    Pod,

};
use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec, vec};
use crate::{
    device::{
        filesystem::{
            config::{FileSystemFeature, VirtioFileSystemConfig},
            fuse::*,
            header::*,
        }, 
        VirtioDeviceError
    },
    queue::VirtQueue,
    transport::{ConfigManager, VirtioTransport},
};


pub struct FileSystemDevice{
    config_manager: ConfigManager<VirtioFileSystemConfig>,
    transport: SpinLock<Box<dyn VirtioTransport>>,
    hiprio_queue: SpinLock<VirtQueue>,
    // notification_queue: SpinLock<VirtQueue>,
    request_queues: Vec<SpinLock<VirtQueue>>,
    hiprio_buffer: DmaStream,
    // notification_buffer: Option<DmaStream>,
    request_buffers: Vec<DmaStream>,
    options: AtomicU64,
}


impl AnyFuseDevice for FileSystemDevice{
    // Util functions
    fn send(&self, concat_req: &[u8], request_queue_idx: usize, locked_request_queue: &mut VirtQueue, readable_len: usize, writeable_start: usize){
        let mut reader = VmReader::from(concat_req);
        let mut writer = self.request_buffers[request_queue_idx].writer().unwrap();
        let len = writer.write(&mut reader);
        self.request_buffers[request_queue_idx].sync(0..len).unwrap();
        let slice_in = DmaStreamSlice::new(&self.request_buffers[request_queue_idx], 0, readable_len);
        let slice_out = DmaStreamSlice::new(&self.request_buffers[request_queue_idx], writeable_start, len);
        locked_request_queue.add_dma_buf(&[&slice_in], &[&slice_out]).unwrap();
        if locked_request_queue.should_notify(){
            locked_request_queue.notify();
        }
    }

    fn init(&self){
        let mut request_queue = self.request_queues[0].disable_irq().lock();

        let headerin = FuseInHeader{
            len: (size_of::<FuseInitIn>() as u32 + size_of::<FuseInHeader>() as u32),
            opcode: FuseOpcode::FuseInit as u32,
            unique: 0,
            nodeid: 0,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        let initin = FuseInitIn {
            major: FUSE_KERNEL_VERSION,
            minor: FUSE_KERNEL_MINOR_VERSION,
            max_readahead: 0,
            flags: FuseInitFlags::FUSE_INIT_EXT.bits() as u32,
            flags2: 0,
            unused: [0u32; 11]
        };
        let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        let initout_buffer = [0u8; size_of::<FuseInitOut>()];

        let headerin_bytes = headerin.as_bytes();
        let initin_bytes = initin.as_bytes();
        let concat_req = [headerin_bytes, initin_bytes, &headerout_buffer, &initout_buffer].concat();
        // Send msg
        let readable_len = size_of::<FuseInHeader>() + size_of::<FuseInitIn>();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, readable_len);

        // let mut reader = VmReader::from(concat_req.as_slice());
        // let mut writer = self.request_buffers[0].writer().unwrap();
        // let len = writer.write(&mut reader);
        // let len_in = size_of::<FuseInitIn>() + size_of::<FuseInHeader>();
        // self.request_buffers[0].sync(0..len).unwrap();
        // let slice_in = DmaStreamSlice::new(&self.request_buffers[0], 0, len_in);
        // let slice_out = DmaStreamSlice::new(&self.request_buffers[0], len_in, len);
        // request_queue.add_dma_buf(&[&slice_in], &[&slice_out]).unwrap();
        // if request_queue.should_notify(){
        //     request_queue.notify();
        // }
    }

    fn handle_init(&self, init_out: FuseInitOut){
        let self_option = self.options.load(Ordering::Relaxed);
        let server_options = ((init_out.flags2 as u64) << 32) | (init_out.flags as u64);
        let options = server_options & self_option;
        self.options.store(options, Ordering::Relaxed);
    }

    fn opendir(&self, nodeid: u64, flags: u32){
        let mut request_queue = self.request_queues[0].disable_irq().lock();

        let headerin = FuseInHeader{
            len: (size_of::<FuseOpenIn>() as u32 + size_of::<FuseInHeader>() as u32),
            opcode: FuseOpcode::FuseOpendir as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        let openin = FuseOpenIn {
            flags: flags,
            open_flags: 0,
        };
        let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        let openout_buffer = [0u8; size_of::<FuseOpenOut>()];

        let headerin_bytes = headerin.as_bytes();
        let openin_bytes = openin.as_bytes();
        let concat_req = [headerin_bytes, openin_bytes, &headerout_buffer, &openout_buffer].concat();
        // Send msg
        let readable_len = size_of::<FuseInHeader>() + size_of::<FuseOpenIn>();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, readable_len);

        // let mut reader = VmReader::from(concat_req.as_slice());
        // let mut writer = self.request_buffers[0].writer().unwrap();
        // let len = writer.write(&mut reader);
        // let len_in = size_of::<FuseOpenIn>() + size_of::<FuseInHeader>();
        // self.request_buffers[0].sync(0..len).unwrap();
        // let slice_in = DmaStreamSlice::new(&self.request_buffers[0], 0, len_in);
        // let slice_out = DmaStreamSlice::new(&self.request_buffers[0], len_in, len);
        // request_queue.add_dma_buf(&[&slice_in], &[&slice_out]).unwrap();
        // if request_queue.should_notify(){
        //     request_queue.notify();
        // }
    }

    fn readdir(&self, nodeid: u64, fh: u64, offset: u64, size: u32){
        let mut request_queue = self.request_queues[0].disable_irq().lock();
        
        let headerin = FuseInHeader{
            len: (size_of::<FuseReadIn>() as u32 + size_of::<FuseInHeader>() as u32),
            opcode: FuseOpcode::FuseReaddir as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        let readin = FuseReadIn {
            fh: fh,
            offset: offset,
            size: size,
            read_flags: 0,
            lock_owner: 0,
            flags: 0,
            padding: 0,
        };
        let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        let readout_buffer = vec![0u8; size as usize];
        
        let headerin_bytes = headerin.as_bytes();
        let readin_bytes = readin.as_bytes();
        let concat_req = [headerin_bytes, readin_bytes, &headerout_buffer, &readout_buffer].concat();

        // Send msg
        let readable_len = size_of::<FuseInHeader>() + size_of::<FuseReadIn>();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, readable_len);

        // let mut reader = VmReader::from(concat_req.as_slice());
        // let mut writer = self.request_buffers[0].writer().unwrap();
        // let len = writer.write(&mut reader);
        // let len_in = size_of::<FuseReadIn>() + size_of::<FuseInHeader>();
        // self.request_buffers[0].sync(0..len).unwrap();
        // let slice_in = DmaStreamSlice::new(&self.request_buffers[0], 0, len_in);
        // let slice_out = DmaStreamSlice::new(&self.request_buffers[0], len_in, len);
        // request_queue.add_dma_buf(&[&slice_in], &[&slice_out]).unwrap();
        // if request_queue.should_notify(){
        //     request_queue.notify();
        // }
    }

    fn mkdir(&self, nodeid: u64, mode: u32, umask: u32, name: &str){
        let mut request_queue = self.request_queues[0].disable_irq().lock();

        let mkdirin = FuseMkdirIn {
            mode: mode,
            umask: umask,
        };
        let prepared_name = fuse_pad_str(name, true);
        let headerin = FuseInHeader{
            len: (size_of::<FuseInHeader>() as u32 + size_of::<FuseMkdirIn>() as u32 + name.len() as u32 + 1),
            opcode: FuseOpcode::FuseMkdir as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        let entryout_buffer = [0u8; size_of::<FuseEntryOut>()];

        let headerin_bytes = headerin.as_bytes();
        let mkdirin_bytes = mkdirin.as_bytes();
        let prepared_name_bytes = prepared_name.as_slice(); 
        let concat_req = [headerin_bytes, mkdirin_bytes, prepared_name_bytes, &headerout_buffer, &entryout_buffer].concat();
        
        //Send msg
        let readable_len = size_of::<FuseInHeader>() + size_of::<FuseMkdirIn>() + prepared_name.len();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, readable_len);
    }

    fn rmdir(&self, nodeid: u64, name: &str){
        let mut request_queue = self.request_queues[0].disable_irq().lock();

        // let mkdirin = FuseMkdirIn {
        //     mode: mode,
        //     umask: umask,
        // };
        let prepared_name = fuse_pad_str(name, true);
        let headerin = FuseInHeader{
            len: (size_of::<FuseInHeader>() as u32 + name.len() as u32 + 1),
            opcode: FuseOpcode::FuseRmdir as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        // let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        // let entryout_buffer = [0u8; size_of::<FuseEntryOut>()];

        let headerin_bytes = headerin.as_bytes();
        // let mkdirin_bytes = mkdirin.as_bytes();
        let prepared_name_bytes = prepared_name.as_slice(); 
        let concat_req = [headerin_bytes, prepared_name_bytes].concat();
        
        //Send msg
        let readable_len = size_of::<FuseInHeader>() + prepared_name.len();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, readable_len);
    }

    fn unlink(&self, nodeid: u64, name: &str){
        let mut request_queue = self.request_queues[0].disable_irq().lock();

        // let mkdirin = FuseMkdirIn {
        //     mode: mode,
        //     umask: umask,
        // };
        let prepared_name = fuse_pad_str(name, true);
        let headerin = FuseInHeader{
            len: (size_of::<FuseInHeader>() as u32 + name.len() as u32 + 1),
            opcode: FuseOpcode::FuseUnlink as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        // let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        // let entryout_buffer = [0u8; size_of::<FuseEntryOut>()];

        let headerin_bytes = headerin.as_bytes();
        // let mkdirin_bytes = mkdirin.as_bytes();
        let prepared_name_bytes = prepared_name.as_slice(); 
        let concat_req = [headerin_bytes, prepared_name_bytes].concat();
        
        //Send msg
        let readable_len = size_of::<FuseInHeader>() + prepared_name.len();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, readable_len);
    }

    fn link(&self, nodeid: u64, oldnodeid: u64, name: &str){
        let mut request_queue = self.request_queues[0].disable_irq().lock();

        let linkin = FuseLinkIn {
            oldnodeid: oldnodeid,
        };

        let prepared_name = fuse_pad_str(name, true);
        let headerin = FuseInHeader{
            len: (size_of::<FuseInHeader>() as u32 + size_of::<FuseLinkIn>() as u32 + name.len() as u32 + 1),
            opcode: FuseOpcode::FuseLink as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        let entryout_buffer = [0u8; size_of::<FuseEntryOut>()];

        let headerin_bytes = headerin.as_bytes();
        let linkin_bytes = linkin.as_bytes();
        let prepared_name_bytes = prepared_name.as_slice(); 
        let concat_req = [headerin_bytes, linkin_bytes, prepared_name_bytes, &headerout_buffer, &entryout_buffer].concat();
        
        //Send msg
        let readable_len = size_of::<FuseInHeader>() + size_of::<FuseLinkIn>() + prepared_name.len();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, readable_len);
    }


    fn statfs(&self, nodeid: u64){
        let mut request_queue = self.request_queues[0].disable_irq().lock();

        // let prepared_name = fuse_pad_str(name, true);
        let headerin = FuseInHeader{
            len: (size_of::<FuseInHeader>() as u32),
            opcode: FuseOpcode::FuseStatfs as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        let kstatfsout_buffer = [0u8; size_of::<FuseKstatfs>()];

        let headerin_bytes = headerin.as_bytes();
        // let prepared_name_bytes = prepared_name.as_slice(); 
        let concat_req = [headerin_bytes, &headerout_buffer, &kstatfsout_buffer].concat();
        
        //Send msg
        let readable_len = size_of::<FuseInHeader>();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, readable_len);
    }

    fn lookup(&self, nodeid: u64, name: &str){
        let mut request_queue = self.request_queues[0].disable_irq().lock();
        let prepared_name = fuse_pad_str(name, true);
        let headerin = FuseInHeader{
            len: (size_of::<FuseInHeader>() as u32 + name.len() as u32 + 1),
            opcode: FuseOpcode::FuseLookup as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        let entryout_buffer = vec![0u8; size_of::<FuseEntryOut>()];

        let headerin_bytes = headerin.as_bytes();
        let prepared_name_bytes = prepared_name.as_slice();
        let concat_req = [headerin_bytes, prepared_name_bytes, &headerout_buffer, &entryout_buffer].concat();

        // Send msg
        let readable_len = size_of::<FuseInHeader>() + prepared_name.len();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, readable_len);
    }

    fn open(&self, nodeid: u64, flags: u32){
        let mut request_queue = self.request_queues[0].disable_irq().lock();
        let headerin = FuseInHeader{
            len: (size_of::<FuseInHeader>() as u32 + size_of::<FuseOpenIn>() as u32),
            opcode: FuseOpcode::FuseOpen as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        let openin = FuseOpenIn{
            flags: flags,
            open_flags: 0,
        };
        let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        let openout_buffer = [0u8; size_of::<FuseOpenOut>()];

        let headerin_bytes = headerin.as_bytes();
        let openin_bytes = openin.as_bytes();
        let concat_req = [headerin_bytes, openin_bytes, &headerout_buffer, &openout_buffer].concat();
        // Send msg
        let readable_len = size_of::<FuseInHeader>() + size_of::<FuseOpenIn>();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, readable_len);
    }

    fn read(&self, nodeid: u64, fh: u64, offset: u64, size: u32){
        let mut request_queue = self.request_queues[0].disable_irq().lock();

        let headerin = FuseInHeader{
            len: (size_of::<FuseReadIn>() as u32 + size_of::<FuseInHeader>() as u32),
            opcode: FuseOpcode::FuseRead as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        let readin = FuseReadIn {
            fh: fh,
            offset: offset,
            size: size,
            read_flags: 0,
            lock_owner: 0,
            flags: 0,
            padding: 0,
        };
        let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        let readout_buffer = vec![0u8; size as usize];

        let headerin_bytes = headerin.as_bytes();
        let readin_bytes = readin.as_bytes();
        let concat_req = [headerin_bytes, readin_bytes, &headerout_buffer, &readout_buffer].concat();

        // Send msg
        let readable_len = size_of::<FuseInHeader>() + size_of::<FuseReadIn>();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, readable_len);
    }

    /// 
    /// Notice: Virtiofsd device will read all device-readable data claimed in descriptor table,
    /// hence the size of claimed device-readable part need to exactly be header size + write header size + original data size
    fn write(&self, nodeid: u64, fh: u64, offset: u64, data: &str){
        let mut request_queue = self.request_queues[0].disable_irq().lock();

        let prepared_data = fuse_pad_str(data, false);
        let writein = FuseWriteIn{
            fh: fh,
            offset: offset,
            size: data.len() as u32,
            write_flags: FUSE_WRITE_LOCKOWNER,
            lock_owner: 0,
            flags: 0,
            padding: 0,
        };
        let headerin = FuseInHeader{
            len: size_of::<FuseInHeader>() as u32 + size_of::<FuseWriteIn>() as u32 + data.len() as u32,
            opcode: FuseOpcode::FuseWrite as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        let writeout_buffer = [0u8; size_of::<FuseWriteOut>()];

        let prepared_data_bytes = prepared_data.as_slice();
        let writein_bytes = writein.as_bytes();
        let headerin_bytes = headerin.as_bytes();
        let concat_req = [headerin_bytes, writein_bytes, prepared_data_bytes, &headerout_buffer, &writeout_buffer].concat();

        // Send msg
        let header_len = size_of::<FuseInHeader>() + size_of::<FuseWriteIn>();
        let readable_len = header_len + data.len();
        let writeable_start = header_len + prepared_data.len();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, writeable_start);

    }

    fn getattr(&self, nodeid: u64, flags: u32, fh: u64){
        let mut request_queue = self.request_queues[0].disable_irq().lock();
        let headerin = FuseInHeader{
            len: (size_of::<FuseInHeader>() as u32 + size_of::<FuseGetattrIn>() as u32),
            opcode: FuseOpcode::FuseGetattr as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        let getattrin = FuseGetattrIn{
            flags: flags,
            dummy: 0,
            fh: fh,
        };
        let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        let getattrout_buffer = [0u8; size_of::<FuseAttrOut>()];


        let headerin_bytes = headerin.as_bytes();
        let getattrin_bytes = getattrin.as_bytes();
        let concat_req = [headerin_bytes, getattrin_bytes, &headerout_buffer, &getattrout_buffer].concat();

        // Send msg
        let readable_len = size_of::<FuseInHeader>() + size_of::<FuseGetattrIn>();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, readable_len);
    }


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
    ){
        let mut request_queue = self.request_queues[0].disable_irq().lock();
        let headerin = FuseInHeader{
            len: (size_of::<FuseInHeader>() as u32 + size_of::<FuseSetattrIn>() as u32),
            opcode: FuseOpcode::FuseSetattr as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        let setattrin = FuseSetattrIn{
            valid: valid,
            padding: 0,
            fh: fh,
            size: size,
            lock_owner: lock_owner,
            atime: atime,
            mtime: mtime,
            ctime: ctime,
            atimensec: atimensec,
            mtimensec: mtimensec,
            ctimensec: ctimensec,
            mode: mode,
            unused4: 0,
            uid: uid,
            gid: gid,
            unused5: 0,
        };
        let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        let setattrout_buffer = [0u8; size_of::<FuseAttrOut>()];

        let headerin_bytes = headerin.as_bytes();
        let setattrin_bytes = setattrin.as_bytes();
        let concat_req = [headerin_bytes, setattrin_bytes, &headerout_buffer, &setattrout_buffer].concat();

        // Send msg
        let readable_len = size_of::<FuseInHeader>() + size_of::<FuseSetattrIn>();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, readable_len);
    }

    fn readlink(&self, nodeid: u64, out_buf_size: u32){
        let mut request_queue = self.request_queues[0].disable_irq().lock();
        let headerin = FuseInHeader{
            len: size_of::<FuseInHeader>() as u32,
            opcode: FuseOpcode::FuseReadlink as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        let dataout_buffer = vec![0u8; out_buf_size as usize];

        let headerin_bytes = headerin.as_bytes();
        let dataout_bytes = dataout_buffer.as_slice();
        let concat_req = [headerin_bytes, &headerout_buffer, dataout_bytes].concat();

        // Send msg
        let readable_len = size_of::<FuseInHeader>();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, readable_len);
    }

    fn copyfilerange(&self, nodeid: u64, fh_in: u64, off_in: u64, nodeid_out: u64, fh_out: u64, off_out: u64, len: u64, flags: u64){
        let mut request_queue = self.request_queues[0].disable_irq().lock();

        let copyfilerangein = FuseCopyfilerangeIn{
            fh_in: fh_in,
            off_in: off_in,
            nodeid_out: nodeid_out,
            fh_out: fh_out,
            off_out: off_out,
            len: len,
            flags: flags,
        };
        let headerin = FuseInHeader{
            len: size_of::<FuseInHeader>() as u32 + size_of::<FuseCopyfilerangeIn>() as u32,
            opcode: FuseOpcode::FuseCopyFileRange as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        let writeout_buffer = [0u8; size_of::<FuseWriteOut>()];

        // let prepared_data_bytes = prepared_data.as_slice();
        let copyfilerangein_bytes = copyfilerangein.as_bytes();
        let headerin_bytes = headerin.as_bytes();
        let concat_req = [headerin_bytes, copyfilerangein_bytes, &headerout_buffer, &writeout_buffer].concat();

        // Send msg
        let header_len = size_of::<FuseInHeader>() + size_of::<FuseCopyfilerangeIn>();
        let readable_len = header_len ;
        let writeable_start = header_len;
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, writeable_start);

    }


    fn symlink(&self, nodeid: u64, name: &str, link_name: &str){
        let mut request_queue = self.request_queues[0].disable_irq().lock();
        let headerin = FuseInHeader{
            len: size_of::<FuseInHeader>() as u32 + name.len() as u32 + link_name.len() as u32 + 2,
            opcode: FuseOpcode::FuseSymlink as u32,
            unique: 0,
            nodeid: nodeid,
            uid: 0,
            gid: 0,
            pid: 0,
            total_extlen: 0,
            padding: 0,
        };
        let concat_name = [name, "\0", link_name].concat();
        let prepared_concat_name = fuse_pad_str(concat_name.as_str(), true);
        let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
        let dataout_buffer = [0u8; size_of::<FuseEntryOut>()];

        let headerin_bytes = headerin.as_bytes();
        let prepared_concat_name_bytes = prepared_concat_name.as_slice();

        let concat_req = [headerin_bytes, prepared_concat_name_bytes, &headerout_buffer, &dataout_buffer].concat();

        // Send msg
        let header_len = size_of::<FuseInHeader>();
        let readable_len = header_len + name.len() + link_name.len() + 2;
        let writeable_start = header_len + prepared_concat_name_bytes.len();
        self.send(concat_req.as_slice(),0, &mut (*request_queue), readable_len, writeable_start);
    }
}


impl FileSystemDevice{
    pub fn negotiate_feature(features: u64) -> u64{
        // notification queue is not supported now
        let mut features = FileSystemFeature::from_bits_truncate(features);
        features.remove(FileSystemFeature::VIRTIO_FS_F_NOTIFICATION);
        features.bits()
    }

    pub fn init(mut transport: Box<dyn VirtioTransport>) -> Result<(), VirtioDeviceError>{
        
        let config_manager = VirtioFileSystemConfig::new_manager(transport.as_ref());
        let fs_config: VirtioFileSystemConfig = config_manager.read_config();
        early_print!("virtio_filesystem_config_notify_buf_size = {:?}\n", fs_config.notify_buf_size);
        early_print!("virtio_filesystem_config_num_request_queues = {:?}\n", fs_config.num_request_queues);
        early_print!("virtio_filesystem_config_tag = {:?}\n", fs_config.tag);

        const HIPRIO_QUEUE_INDEX: u16 = 0;
        // const NOTIFICATION_QUEUE_INDEX: u16 = 1;
        const REQUEST_QUEUE_BASE_INDEX: u16 = 1;
        let hiprio_queue= SpinLock::new(VirtQueue::new(HIPRIO_QUEUE_INDEX, 2, transport.as_mut()).unwrap());
        // let notification_queue= SpinLock::new(VirtQueue::new(NOTIFICATION_QUEUE_INDEX, 2, transport.as_mut()).unwrap());
        let mut request_queues = Vec::new();
        for i in 0..fs_config.num_request_queues{
            request_queues.push(SpinLock::new(VirtQueue::new(REQUEST_QUEUE_BASE_INDEX + (i as u16), 4, transport.as_mut()).unwrap()))
        }

        let hiprio_buffer = {
            let vm_segment = FrameAllocOptions::new().alloc_segment(3).unwrap();
            DmaStream::map(vm_segment.into(), DmaDirection::Bidirectional, false).unwrap()
        };
        
        let mut request_buffers = Vec::new();
        for _ in 0..fs_config.num_request_queues{
            let request_buffer = {
                let vm_segment = FrameAllocOptions::new().alloc_segment(3).unwrap();
                DmaStream::map(vm_segment.into(), DmaDirection::Bidirectional, false).unwrap()
            };
            request_buffers.push(request_buffer);
        };

        let device = Arc::new(Self{
            config_manager: config_manager,
            transport: SpinLock::new(transport),
            hiprio_queue: hiprio_queue,
            // notification_queue: notification_queue,
            request_queues: request_queues,
            hiprio_buffer: hiprio_buffer,
            request_buffers: request_buffers,
            options: AtomicU64::new(FuseInitFlags::empty().bits()),
        });
        let handle_request = {
            let device = device.clone();
            move |_: &TrapFrame| device.handle_recv_irq()
        };
        let config_space_change = |_: &TrapFrame| early_print!("Config Changed\n");
        let mut transport = device.transport.disable_irq().lock();
        transport
            .register_queue_callback(REQUEST_QUEUE_BASE_INDEX + 0, Box::new(handle_request), false)
            .unwrap();
        transport
            .register_cfg_callback(Box::new(config_space_change))
            .unwrap();
        transport.finish_init();
        drop(transport);
        
        // device.init();
        test_device(&device);

        Ok(())
    }

    fn handle_recv_irq(&self){
        let mut request_queue = self.request_queues[0].disable_irq().lock();
        let Ok((_, len)) = request_queue.pop_used() else {
            return;
        };
        self.request_buffers[0].sync(0..len as usize).unwrap();
        let mut reader = self.request_buffers[0].reader().unwrap();
        let headerin = reader.read_val::<FuseInHeader>().unwrap();
        // Remove Data_in
        let trash_len = headerin.len as usize - size_of::<FuseInHeader>();
        let pad_trash_len = trash_len + ((8 - (trash_len & 0x7)) & 0x7); //pad to multiple of 8 bytes
        let mut trash_vec = vec![0u8; pad_trash_len];
        let mut trash_writer = VmWriter::from(trash_vec.as_mut_slice());
        trash_writer.write(&mut reader);
        match to_opcode(headerin.opcode).unwrap() {
            FuseOpcode::FuseInit => {
                let _ = reader.read_val::<FuseOutHeader>().unwrap();
                let dataout = reader.read_val::<FuseInitOut>().unwrap();
                early_print!("Received Init Msg\n");
                early_print!("major:{:?}\n", dataout.major);
                early_print!("minor:{:?}\n", dataout.minor);
                early_print!("flags:{:?}\n", dataout.flags);
                self.handle_init(dataout);
            },
            FuseOpcode::FuseReaddir => {
                let headerout = reader.read_val::<FuseOutHeader>().unwrap();
                let readdir_out = FuseReaddirOut::read_dirent(&mut reader, headerout);
                early_print!("Readdir response received: len={:?}, error={:?}\n", headerout.len, headerout.error);
                for dirent_name in readdir_out.dirents{
                    let dirent = dirent_name.dirent;
                    let name = String::from_utf8(dirent_name.name).unwrap();
                    early_print!("Readdir response received: inode={:?}, off={:?}, namelen={:?}, type={:?}, filename={:?}\n", 
                        dirent.ino, dirent.off, dirent.namelen, dirent.type_, name);
                }
            },
            FuseOpcode::FuseOpendir => {
                let headerout = reader.read_val::<FuseOutHeader>().unwrap();
                let dataout = reader.read_val::<FuseOpenOut>().unwrap();
                early_print!("Opendir response received: len={:?}, error={:?}\n", headerout.len, headerout.error);
                early_print!("Opendir response received: fh={:?}, open_flags={:?}, backing_id={:?}\n", dataout.fh, dataout.open_flags, dataout.backing_id);                
            },
            FuseOpcode::FuseMkdir => {
                let headerout = reader.read_val::<FuseOutHeader>().unwrap();
                let dataout = reader.read_val::<FuseEntryOut>().unwrap();
                early_print!("Mkdir response received: len={:?}, error={:?}\n", headerout.len, headerout.error);
                early_print!("Mkdir response received: nodeid={:?}, generation={:?}, entry_valid={:?}, attr_valid={:?}\n", 
                    dataout.nodeid, dataout.generation, dataout.entry_valid, dataout.attr_valid);
            },
            FuseOpcode::FuseLink => {
                let headerout = reader.read_val::<FuseOutHeader>().unwrap();
                let dataout = reader.read_val::<FuseEntryOut>().unwrap();
                early_print!("Link response received: len={:?}, error={:?}\n", headerout.len, headerout.error);
                early_print!("Link response received: nodeid={:?}, generation={:?}, entry_valid={:?}, attr_valid={:?}\n", 
                    dataout.nodeid, dataout.generation, dataout.entry_valid, dataout.attr_valid);
            },
            FuseOpcode::FuseStatfs => {
                let headerout = reader.read_val::<FuseOutHeader>().unwrap();
                let dataout = reader.read_val::<FuseKstatfs>().unwrap();
                early_print!("Statfs response received: len={:?}, error={:?}\n", headerout.len, headerout.error);
                early_print!("Statfs response received: blocks={:?}, bfree={:?}, bavail={:?}, files={:?}, ffree={:?}, bsize={:?}, namelen={:?}, frsize={:?}, padding={:?}, spare={:?}\n", 
                    dataout.blocks, dataout.bfree, dataout.bavail, dataout.files, dataout.ffree, dataout.bsize, dataout.namelen, dataout.frsize, dataout.padding, dataout.spare);
            },
            FuseOpcode::FuseLookup => {
                let headerout = reader.read_val::<FuseOutHeader>().unwrap();
                let dataout = reader.read_val::<FuseEntryOut>().unwrap();
                early_print!("Lookup response received: len={:?}, error={:?}\n", headerout.len, headerout.error);
                early_print!("Lookup response received: nodeid={:?}, generation={:?}, entry_valid={:?}, attr_valid={:?}\n", 
                    dataout.nodeid, dataout.generation, dataout.entry_valid, dataout.attr_valid);
            },
            FuseOpcode::FuseOpen => {
                let headerout = reader.read_val::<FuseOutHeader>().unwrap();
                let dataout = reader.read_val::<FuseOpenOut>().unwrap();
                early_print!("Open response received: len={:?}, error={:?}\n", headerout.len, headerout.error);
                early_print!("Open response received: fh={:?}, open_flags={:?}, backing_id={:?}\n", dataout.fh, dataout.open_flags, dataout.backing_id);   
            },
            FuseOpcode::FuseRead => {
                let headerout = reader.read_val::<FuseOutHeader>().unwrap();
                early_print!("Read response received: len={:?}, error={:?}\n", headerout.len, headerout.error);
                if headerout.len > size_of::<FuseOutHeader>() as u32 {
                    let data_len = headerout.len - size_of::<FuseOutHeader>() as u32;
                    let mut dataout_buf = vec![0u8; data_len as usize];
                    let mut writer = VmWriter::from(dataout_buf.as_mut_slice());
                    writer.write(&mut reader);
                    let data_utf8 = String::from_utf8(dataout_buf).unwrap();
                    early_print!("Read response received: data={:?}\n", data_utf8);
                }
            }
            FuseOpcode::FuseWrite => {
                let headerout = reader.read_val::<FuseOutHeader>().unwrap();
                early_print!("Write response received: len={:?}, error={:?}\n", headerout.len, headerout.error);
                if headerout.len > size_of::<FuseOutHeader>() as u32 {
                    let writeout = reader.read_val::<FuseWriteOut>().unwrap();
                    early_print!("Write response received: size={:?}\n", writeout.size);
                }
            },
            FuseOpcode::FuseGetattr => {
                let headerout = reader.read_val::<FuseOutHeader>().unwrap();
                early_print!("Getattr response received: len={:?}, error={:?}\n", headerout.len, headerout.error);
                if headerout.len > size_of::<FuseOutHeader>() as u32 {
                    let attrout = reader.read_val::<FuseAttrOut>().unwrap();
                    early_print!("Getattr response received: attr_valid={:?}\n", attrout.attr_valid);
                    early_print!("Getattr response received: flags={:?}, ino={:?}, mode={:?}\n", attrout.attr.flags, attrout.attr.ino, attrout.attr.mode);
                }
            },
            FuseOpcode::FuseSetattr => {
                let headerout = reader.read_val::<FuseOutHeader>().unwrap();
                early_print!("Setattr response received: len={:?}, error={:?}\n", headerout.len, headerout.error);
                if headerout.len > size_of::<FuseOutHeader>() as u32 {
                    let attrout = reader.read_val::<FuseAttrOut>().unwrap();
                    early_print!("Setattr response received: attr_valid={:?}\n", attrout.attr_valid);
                    early_print!("Setattr response received: flags={:?}, ino={:?}, mode={:?}\n", attrout.attr.flags, attrout.attr.ino, attrout.attr.mode);
                }
            },
            FuseOpcode::FuseReadlink => {
                let headerout = reader.read_val::<FuseOutHeader>().unwrap();
                early_print!("Readlink response received: len={:?}, error={:?}\n", headerout.len, headerout.error);
                
                if headerout.len > size_of::<FuseOutHeader>() as u32 {
                    let mut dataout_buffer = vec![0u8; headerout.len as usize - size_of::<FuseOutHeader>()];
                    let mut writer = VmWriter::from(dataout_buffer.as_mut_slice());
                    writer.write(&mut reader);
                    let symlink = String::from_utf8(dataout_buffer).unwrap();
                    early_print!("Readlink response received: symlink={:?}\n", symlink);
                }
            },
            FuseOpcode::FuseSymlink => {
                let headerout = reader.read_val::<FuseOutHeader>().unwrap();
                early_print!("Symlink response received: len={:?}, error={:?}\n", headerout.len, headerout.error);

                if headerout.len > size_of::<FuseOutHeader>() as u32 {
                    let dataout = reader.read_val::<FuseEntryOut>().unwrap();
                    early_print!("Symlink response received: nodeid={:?}, generation={:?}, entry_valid={:?}, attr_valid={:?}\n", 
                        dataout.nodeid, dataout.generation, dataout.entry_valid, dataout.attr_valid);
                }
            }
            FuseOpcode::FuseCopyFileRange => {
                let headerout = reader.read_val::<FuseOutHeader>().unwrap();
                early_print!("CopyFileRange response received: len={:?}, error={:?}\n", headerout.len, headerout.error);
                if headerout.len > size_of::<FuseOutHeader> as u32 {
                    let writeout = reader.read_val::<FuseWriteOut>().unwrap();
                    early_print!("CopyFileRange response received: size={:?}\n", writeout.size);
                }
            }
            _ => {
            }
        };
        drop(request_queue);
        test_device(&self);
    }
}


static TEST_COUNTER: RwLock<u32> = RwLock::new(0);
pub fn test_device(device: &FileSystemDevice){
    let test_counter = {
        let mut test_counter = TEST_COUNTER.write();
        *test_counter += 1;
        *test_counter
    };


    match test_counter{
        // 1 => device.opendir(1,0),
        // 2 => device.readdir(1,0,0,128),
        // 3 => device.lookup(1, "testl"),
        // 3 => device.mkdir(1, 0o755, 0o777, "MkdirTest"),
        // 3 => device.lookup(1, "testh"),
        // 4 => device.open(2, 2),
        // 5 => device.write(2, 1, 0, "Hello from Guest!!\n"),
        // 6 => device.read(2, 1, 0, 128),
        // 7 => device.getattr(2, FUSE_GETATTR_FH, 1),
        // 8 => device.setattr(
        //     2,
        //     FuseSetattrValid::MODE.bits(), 
        //     1,
        //     0, 
        //     0, 
        //     0, 
        //     0 , 
        //     0 , 
        //     0, 
        //     0,
        //     0,
        //     0o100755,
        //     0,
        //     0,
        // ),
        // 4 => device.symlink(1, "test_guest", "testh"),
        // 4 => device.readlink(2, 128),

        // copyfilerange function test
        1 => device.opendir(1,0),
        2 => device.readdir(1,0,0,128),
        3 => device.lookup(1, "testh"),

        4 => device.open(2, 2),
        5 => device.lookup(1, "testg"),
        6 => device.open(3, 2),
        7 => device.copyfilerange(2, 1, 0, 3, 2, 0, 6, 0),





        // 3 => device.statfs(1),
        // 4 => device.mkdir(1, 0o755, 0o777, "newdir"),
        // 5 => device.readdir(1,0,0,128),
        // 6 => device.statfs(1),


        // 3 => device.lookup(1, "testh"),
        // 4 => device.link(1,2,"newtesth"),

        // 3 => device.unlink(1, "testh"),
        // 4 => device.readdir(1,0,0,128),
        // // 3 => device.mkdir(1, 0o755, 0o777, "MkdirTest"),
        // 4 => device.open(2, 2),
        // 5 => device.write(2, 1, 0, "Hello from Guest!!\n"),
        // 6 => device.read(2, 1, 0, 128),


        // // 7 => device.mkdir(1, 0o755, 0o777, "newtest"),
        // 7 => device.readdir(1,0,0,128),
        // 8 => device.rmdir(1,"hht"),
        // 9 => device.readdir(1,0,0,128),
        _ => ()
    };
}