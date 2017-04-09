extern crate libdrm;

#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json as json;

use std::env;
use std::cmp::Ordering;

use libdrm as drm;

#[derive(Serialize,Debug)]
struct ErrorReply {
    code: i32,
    message: String,
}

#[derive(Serialize)]
struct InfoCommandReply {
    device_ids: Vec<String>,
}

#[derive(Serialize)]
struct AddCommandReply {
    primary_node: String,
}

pub enum Command {
    Info,
    Add,
}

impl Command {
    fn from_str(s: &String) -> Result<Self, String> {
        return match s.as_str() {
            "INFO" => Ok(Command::Info),
            "ADD" => Ok(Command::Add),
            _ => Err(format!("Unsupported command: {}", s)),
        }
    }
}

fn get_device_id(d: &drm::Device) -> String {
    return match d.info {
        drm::DeviceInfo::Pci {bus, dev: _} => format!("{}:{}:{}.{}", bus.domain, bus.bus, bus.dev, bus.func),
        drm::DeviceInfo::Usb {bus, dev: _} => format!("{}:{}", bus.bus, bus.dev),
        drm::DeviceInfo::Platform {ref bus, dev: _} => bus.fullname.clone(),
        drm::DeviceInfo::Host1x {ref bus, dev: _} => bus.fullname.clone(),
    }
}

fn info() -> Result<InfoCommandReply, ErrorReply> {
    let devices = drm::get_devices();
    let mut device_ids : Vec<String> = Vec::with_capacity(devices.len());
    for d in devices {
        device_ids.push(get_device_id(&d));
    }
    return Ok(InfoCommandReply{ device_ids: device_ids })
}

fn add(device_id: &String) -> Result<AddCommandReply, ErrorReply> {
    let devices = drm::get_devices();
    let mut device = None;
    for d in devices {
        if device_id.cmp(&get_device_id(&d)) == Ordering::Equal {
            device = Some(d)
        }
    };
    match device {
        Some(d) => return Ok(AddCommandReply{
            primary_node: d.nodes.primary.unwrap()
        }),
        None => return Err(ErrorReply{
            code: 1,
            message: format!("Could not find node for device_id: {}", device_id)
        }),
    }
}

fn main() {
    let command = Command::from_str(&env::var("CDI_COMMAND").unwrap()).unwrap();

    if !drm::available() {
        println!("{}", json::to_string(&ErrorReply{
            code: 2,
            message: "DRM unavailable!".to_string()
        }).unwrap());
        std::process::exit(1);
    }

    match command {
        Command::Info => {
            match info() {
                Ok(reply) => println!("{}", json::to_string(&reply).unwrap()),
                Err(error) => {
                    println!("{}", json::to_string(&error).unwrap());
                    std::process::exit(1);
                }
            }
        },
        Command::Add => {
            match add(&env::var("CDI_DEVICE_ID").unwrap()) {
                Ok(reply) => println!("{}", json::to_string(&reply).unwrap()),
                Err(error) => {
                    println!("{}", json::to_string(&error).unwrap());
                    std::process::exit(1);
                }
            }
        },
    };
}
