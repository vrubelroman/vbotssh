pub mod disks;
pub mod docker;
pub mod local;
pub mod remote;

use anyhow::Result;

use crate::model::{HostDescriptor, HostInfo};

pub trait HostCollector: Send {
    fn descriptor(&self) -> HostDescriptor;
    fn collect(&mut self) -> Result<HostInfo>;
}
