use std::{collections::BTreeSet, iter::once, time::Duration};

use anyhow::{bail, Context};
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

    let mut handle = device.open()?;
    println!(
        "Kernel driver? => {:?}",
        handle.kernel_driver_active(interface_desc.interface_number())
    );

    handle.set_active_configuration(conf_desc.number())?;
    let iface_num = interface_desc.interface_number();
    handle.claim_interface(iface_num)?;
    handle.set_alternate_setting(iface_num, interface_desc.setting_number())?;

    let addresses = interface_desc
        .endpoint_descriptors()
        .map(|e| e.address())
        .collect::<BTreeSet<_>>();
    if !addresses.contains(&0x04) {
        bail!("Required endpoints were not found!");
    }

    let timeout = Duration::from_secs(1);
    // Enable only channel 3 by manipulating register 0x00
    handle.write_bulk(0x04, &bytes_to_bits([0b_0000_0000, 0b_1000_0000]), timeout)?;
    // Set the frequency to about 0.6 MHz by manipulating register 0x04
    handle.write_bulk(
        0x04,
        &bytes_to_bits(once(0b_0000_0100).chain(0x004EA4A9_u32.to_be_bytes())),
        timeout,
    )?;

    Ok(())
}

fn bytes_to_bits(v: impl IntoIterator<Item = u8>) -> Vec<u8> {
    v.into_iter()
        .flat_map(|v| (0..8).map(move |i| if v & 1 << i > 0 { 1 } else { 0 }))
        .collect()
}
