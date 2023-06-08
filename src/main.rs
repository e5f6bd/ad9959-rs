use std::{collections::BTreeSet, iter::once, time::Duration};

use anyhow::{bail, Context};
use rusb::{DeviceList, UsbOption};

fn main() -> anyhow::Result<()> {
    macro_rules! sleep {
        () => {
            println!("line = {}", line!());
            std::thread::sleep(std::time::Duration::from_secs(1));
        }
    }

    let context = rusb::Context::with_options(&[UsbOption::use_usbdk()])?;
    let (dev_desc, device) = DeviceList::new_with_context(context)?
        .iter()
        .filter_map(|d| Some((d.device_descriptor().ok()?, d)))
        .find(|(desc, device)| {
            desc.vendor_id() == 0x0456 && desc.product_id() == 0xee25 && device.port_number() == 1
        })
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

    dbg!(device.address());
    dbg!(device.port_number());
    let _ = dbg!(device.port_numbers());

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
    println!("{:?}", addresses);
    if ![0x01, 0x04, 0x88].iter().all(|x| addresses.contains(x)) {
        bail!("Required endpoints were not found!");
    }

    let timeout = Duration::from_secs(1);
    // Enable only channel 2 by manipulating register 0x00
    handle.write_bulk(0x04, &bytes_to_bits([0b_0000_0000, 0b_0100_0000]), timeout)?;
    sleep!();

    // Set the frequency to about 0.6 MHz by manipulating register 0x04
    handle.write_bulk(
        0x04,
        &bytes_to_bits(once(0b_0000_0100).chain(0x004EA4A9_u32.to_be_bytes())),
        timeout,
    )?;
    sleep!();

    // Load I/O and update I/O
    handle.write_bulk(0x01, &[0x0C, 0x00], timeout)?;
    sleep!();
    handle.write_bulk(0x01, &[0x0C, 0x10], timeout)?;
    sleep!();

    // let register = 0x04;
    // let mut buf = [0; 0x20];
    // // Start readback mode
    // handle.write_bulk(0x01, &[0x07, 0x00, buf.len() as u8], timeout)?;
    // // Send "read register 0x00" message
    // handle.write_bulk(0x04, &bytes_to_bits([0b_1000_0000 | register]), timeout)?;
    // // Wait for the readback
    // let res = handle.read_bulk(0x88, &mut buf, timeout)?;
    // println!("Read {res} bytes: {buf:?}");
    // // End readback mode (not sure if it's really needed)
    // handle.write_bulk(0x01, &[0x04, 0x00], timeout)?;

    Ok(())
}

fn bytes_to_bits(v: impl IntoIterator<Item = u8>) -> Vec<u8> {
    v.into_iter()
        .flat_map(|v| (0..8).map(move |i| if v & 1 << i > 0 { 1 } else { 0 }))
        .collect()
}
