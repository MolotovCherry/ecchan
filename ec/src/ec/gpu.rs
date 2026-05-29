use std::sync::{
    OnceLock,
    atomic::{AtomicBool, Ordering},
};

use nvml_wrapper::{
    Device, Nvml,
    struct_wrappers::device::{MemoryInfo, Utilization},
};

use super::*;

fn get_nvml(pci_bus_id: Option<&str>) -> Result<&'static Device<'static>> {
    static ATTEMPTED: AtomicBool = AtomicBool::new(false);
    static NVML: OnceLock<Nvml> = OnceLock::new();
    static DEVICE: OnceLock<Device> = OnceLock::new();

    if let Some(d) = DEVICE.get() {
        return Ok(d);
    }

    let attempted = ATTEMPTED.swap(true, Ordering::Relaxed);
    if attempted {
        whatever!("GPU is unsupported");
    }

    let Some(pci_bus_id) = pci_bus_id else {
        log::error!("expected pci_bus_id to be Some");
        whatever!("expected pci_bus_id to be Some");
    };

    let nvml = match Nvml::init() {
        Ok(n) => n,
        Err(e) => {
            log::error!(
                "failed trying to init nvidia gpu; if you don't use nvidia, this is not an error: {e}"
            );
            return Err(e).context(NvmlSnafu);
        }
    };

    NVML.set(nvml).unwrap();

    let nvml = NVML.get().unwrap();

    let device = match nvml.device_by_pci_bus_id(pci_bus_id) {
        Ok(d) => d,
        Err(e) => {
            log::error!("failed trying to get nvidia gpu device: {e}");
            return Err(e).context(NvmlSnafu);
        }
    };

    DEVICE.set(device).unwrap();

    Ok(DEVICE.get().unwrap())
}

impl Ec {
    pub fn gpu_init(&self, pci_bus_id: &str) -> Result<()> {
        get_nvml(Some(pci_bus_id)).map(|_| ())
    }

    pub fn gpu_memory_info(&self) -> Result<MemoryInfo> {
        let device = get_nvml(None)?;
        device.memory_info().context(NvmlSnafu)
    }

    pub fn gpu_utilization_rates(&self) -> Result<Utilization> {
        let device = get_nvml(None)?;
        device.utilization_rates().context(NvmlSnafu)
    }
}
