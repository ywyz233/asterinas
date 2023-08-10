use crate::prelude::*;

use super::*;

/// Same major number with Linux.
const PTMX_MAJOR_NUM: u32 = 5;
/// Same minor number with Linux.
const PTMX_MINOR_NUM: u32 = 2;

/// Ptmx is the multiplexing master of devpts.
///
/// Every time the multiplexing master is opened, a new instance of pty master inode is returned
/// and an corresponding pty slave inode is also created.
pub struct Ptmx {
    inner: Inner,
    metadata: Metadata,
    fs: Weak<DevPts>,
}

impl Ptmx {
    pub fn new(sb: &SuperBlock, fs: Weak<DevPts>) -> Arc<Self> {
        let inner = Inner;
        Arc::new(Self {
            metadata: Metadata::new_device(
                PTMX_INO,
                InodeMode::from_bits_truncate(0o666),
                sb,
                &inner,
            ),
            inner,
            fs,
        })
    }

    /// The open method for ptmx.
    ///
    /// Creates a master and slave pair and returns the master inode.
    pub fn open(&self) -> Result<Arc<PtyMasterInode>> {
        let (master, _) = self.devpts().create_master_slave_pair()?;
        Ok(master)
    }

    pub fn devpts(&self) -> Arc<DevPts> {
        self.fs.upgrade().unwrap()
    }

    pub fn device_type(&self) -> DeviceType {
        self.inner.type_()
    }

    pub fn device_id(&self) -> DeviceId {
        self.inner.id()
    }
}

// Many methods are left to do nothing because every time the ptmx is being opened,
// it returns the pty master. So the ptmx can not be used at upper layer.
impl Inode for Ptmx {
    fn len(&self) -> usize {
        self.metadata.size
    }

    fn resize(&self, new_size: usize) {}

    fn metadata(&self) -> Metadata {
        self.metadata.clone()
    }

    fn type_(&self) -> InodeType {
        self.metadata.type_
    }

    fn mode(&self) -> InodeMode {
        self.metadata.mode
    }

    fn set_mode(&self, mode: InodeMode) {}

    fn atime(&self) -> Duration {
        self.metadata.atime
    }

    fn set_atime(&self, time: Duration) {}

    fn mtime(&self) -> Duration {
        self.metadata.mtime
    }

    fn set_mtime(&self, time: Duration) {}

    fn read_page(&self, idx: usize, frame: &VmFrame) -> Result<()> {
        Ok(())
    }

    fn write_page(&self, idx: usize, frame: &VmFrame) -> Result<()> {
        Ok(())
    }

    fn read_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
        Ok(0)
    }

    fn read_direct_at(&self, offset: usize, buf: &mut [u8]) -> Result<usize> {
        Ok(0)
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> Result<usize> {
        Ok(0)
    }

    fn write_direct_at(&self, offset: usize, buf: &[u8]) -> Result<usize> {
        Ok(0)
    }

    fn ioctl(&self, cmd: IoctlCmd, arg: usize) -> Result<i32> {
        Ok(0)
    }

    fn fs(&self) -> Arc<dyn FileSystem> {
        self.devpts()
    }

    fn as_device(&self) -> Option<Arc<dyn Device>> {
        Some(Arc::new(self.inner))
    }
}

#[derive(Clone, Copy)]
struct Inner;

impl Device for Inner {
    fn type_(&self) -> DeviceType {
        DeviceType::CharDevice
    }

    fn id(&self) -> DeviceId {
        DeviceId::new(PTMX_MAJOR_NUM, PTMX_MINOR_NUM)
    }

    fn read(&self, buf: &mut [u8]) -> Result<usize> {
        // do nothing because it should not be used to read.
        Ok(0)
    }

    fn write(&self, buf: &[u8]) -> Result<usize> {
        // do nothing because it should not be used to write.
        Ok(buf.len())
    }
}
