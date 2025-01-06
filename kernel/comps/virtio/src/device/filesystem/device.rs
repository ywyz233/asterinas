use ostd::{
    early_print, mm::{DmaDirection, DmaStream, DmaStreamSlice, FrameAllocOptions, VmReader}, sync::SpinLock, trap::TrapFrame, Pod
};
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use crate::{
    device::{
        filesystem::{
            config::{FileSystemFeature, VirtioFileSystemConfig},
            fuse::*,
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
            request_queues.push(SpinLock::new(VirtQueue::new(REQUEST_QUEUE_BASE_INDEX + (i as u16), 2, transport.as_mut()).unwrap()))
        }

        let hiprio_buffer = {
            let vm_segment = FrameAllocOptions::new().alloc_segment(1).unwrap();
            DmaStream::map(vm_segment.into(), DmaDirection::Bidirectional, false).unwrap()
        };
        
        let mut request_buffers = Vec::new();
        for _ in 0..fs_config.num_request_queues{
            let request_buffer = {
                let vm_segment = FrameAllocOptions::new().alloc_segment(1).unwrap();
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
        
        test_device(device);

        Ok(())
    }

    fn handle_recv_irq(&self){
        let mut request_queue = self.request_queues[0].disable_irq().lock();

        let Ok((_, len)) = request_queue.pop_used() else {
            return;
        };
        self.request_buffers[0].sync(0..len as usize).unwrap();
        let reader = self.request_buffers[0].reader().unwrap();
        // reader.read_once::F
        early_print!("Received Init\n");
    }
}

fn test_device(device: Arc<FileSystemDevice>) {
    let mut request_queue = device.request_queues[0].lock();
    // let request_buffer = device.request_buffers[0].clone();
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
    let headerin_bytes = headerin.as_bytes();
    let initin_bytes = initin.as_bytes();
    let headerout_buffer = [0u8; size_of::<FuseOutHeader>()];
    let initout_bytes = [0u8; 256];
    let concat_req = [headerin_bytes, initin_bytes, &headerout_buffer, &initout_bytes].concat();
    
    // Send msg
    let mut reader = VmReader::from(concat_req.as_slice());
    let mut writer = device.request_buffers[0].writer().unwrap();
    let len = writer.write(&mut reader);
    let len_in = size_of::<FuseInitIn>() + size_of::<FuseInHeader>();
    device.request_buffers[0].sync(0..len).unwrap();
    
    let slice_in = DmaStreamSlice::new(&device.request_buffers[0], 0, len_in);
    let slice_out = DmaStreamSlice::new(&device.request_buffers[0], len_in, len);
    request_queue.add_dma_buf(&[&slice_in], &[&slice_out]).unwrap();

    if request_queue.should_notify(){
        request_queue.notify();
    }
}