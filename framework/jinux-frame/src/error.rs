/// The error type which is returned from the APIs of this crate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Error {
    InvalidArgs,
    NoMemory,
    PageFault,
    AccessDenied,
    IoError,
    NotEnoughResources,
    TimeOut,
}
