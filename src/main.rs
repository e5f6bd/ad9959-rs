use anyhow::Context;
use rusb::{DeviceList, UsbOption};

fn main() -> anyhow::Result<()> {
    let context = rusb::Context::with_options(&[UsbOption::use_usbdk()])?;
    let (dev_desc, device) = DeviceList::new_with_context(context)?
        .iter()
        .filter_map(|d| Some((d.device_descriptor().ok()?, d)))
        .find(|(desc, device)| desc.vendor_id() == 0x0456 && desc.product_id() == 0xee25)
        .context("AD9959 not found")?;
    let conf_desc = device.config_descriptor(0)?;
    let interface = conf_desc
        .interfaces()
        .next()
        .context("Interface not found")?;
    let interface_desc = interface
        .descriptors()
        .next()
        .context("Interface descriptor not found")?;

    let handle = device.open()?;
    println!(
        "Kernel driver? => {:?}",
        handle.kernel_driver_active(interface_desc.interface_number())?
    );

    // for interface in conf_desc.interfaces() {
    //     println!("interface");
    //     for interface_desc in interface.descriptors() {
    //         println!("  interface_desc {interface_desc:?}");
    //         for endpoint_desc in interface_desc.endpoint_descriptors() {
    //             println!("    endpoint_desc address={:?}", endpoint_desc.address());
    //         }
    //     }
    // }

    Ok(())
}
