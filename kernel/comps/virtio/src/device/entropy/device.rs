
use ostd::{
    early_println, mm::{DmaDirection, DmaStream, DmaStreamSlice, FrameAllocOptions, VmReader}, sync::{LocalIrqDisabled, RwLock, SpinLock}, trap::TrapFrame
};
use core::hint::spin_loop;
use alloc::{boxed::Box, fmt::Debug, string::ToString, sync::Arc, vec::Vec};
use crate::{
    device::{console::config::ConsoleFeatures, VirtioDeviceError},
    queue::VirtQueue,
    transport::{ConfigManager, VirtioTransport},
};


pub struct EntropyDevice{
    request_buffer: DmaStream,
    request_queue: SpinLock<VirtQueue>,
    transport: SpinLock<Box<dyn VirtioTransport>>
}

impl EntropyDevice {
    pub fn negotiate_features(features: u64) -> u64 {
        features
    }

    pub fn init(mut transport: Box<dyn VirtioTransport>) -> Result<(), VirtioDeviceError> {
        // Initalize the request virtqueue
        const REQUEST_QUEUE_INDEX: u16 = 0;
        let request_queue = SpinLock::new(VirtQueue::new(REQUEST_QUEUE_INDEX, 1, transport.as_mut()).unwrap());
        // Initalize the request buffer
        let request_buffer = {
            let vm_segment = FrameAllocOptions::new().alloc_segment(1).unwrap();
            DmaStream::map(vm_segment.into(), DmaDirection::FromDevice, false).unwrap()
        };
        // Create device
        let device = Arc::new(Self {
            request_buffer,
            request_queue,
            transport: SpinLock::new(transport),
        });
        // Finish init
        device.transport.lock().finish_init();
        // Test device
        test_device(device);
        Ok(())
    }
}

fn test_device(device: Arc<EntropyDevice>) {
    let mut request_queue = device.request_queue.lock();
    let request_buffer = device.request_buffer.clone();

    let value = request_buffer.reader().unwrap().read_once::<u64>().unwrap();
    early_println!("Before value:{:x}", value);
    request_queue
        .add_dma_buf(&[], &[&DmaStreamSlice::new(&request_buffer, 0, 8)])
        .unwrap();
    if request_queue.should_notify() {
        request_queue.notify();
    }
    while !request_queue.can_pop() {
        spin_loop();
    }
    request_queue.pop_used().unwrap();
    request_buffer.sync(0..8).unwrap();
    let value = request_buffer.reader().unwrap().read_once::<u64>().unwrap();
    early_println!("After value:{:x}", value);
}
   