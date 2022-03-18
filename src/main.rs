#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(improper_ctypes)]

use std::ffi::CStr;
use std::ffi::CString;

use std::fs::File;
use std::io::BufWriter;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::os::raw::c_char;

use clap::Parser;

// include the bindings
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern "C" {
    pub fn get_child(
        udev: *mut udev,
        parent: *mut udev_device,
        subsystem: *const ::std::os::raw::c_char,
    ) -> *mut udev_device;

    pub fn enumerate_usb_mass_storage(udev: *mut udev);

    pub fn get_device_info_by_block(device_path: *const c_char) -> *mut DeviceInfo;
    pub fn get_device_info_by_vp(
        id_vendor: *const c_char,
        id_product: *const c_char,
    ) -> *mut DeviceInfo;
}

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
enum Error {
    Block(&'static str),

    CStringInit(std::ffi::NulError),

    DeviceInfoInit,

    FileOpen(std::io::Error),
    FileRead(std::io::Error),
    FileWrite(std::io::Error),

    Seek(u64),

    UDevInit(&'static str),
    Unmount(i32),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Block(ref str) => write!(f, "{}", str),
            Self::CStringInit(ref err) => write!(f, "Could not create CString: {:?}", err),
            Self::DeviceInfoInit => write!(f, "Could not instantiate DeviceInfo"),
            Self::FileOpen(ref err) => write!(f, "Error opening file: {:?}", err),
            Self::FileRead(ref err) => write!(f, "Error reading file: {:?}", err),
            Self::FileWrite(ref err) => write!(f, "Error writing to file: {:?}", err),
            Self::Seek(ref pos) => write!(f, "Error seeking to position {}", pos),
            Self::UDevInit(ref str) => write!(f, "{}", str),
            Self::Unmount(ref no) => {
                if *no == 1 {
                    write!(f, "root permissions required to unmount device ({})", no)
                } else {
                    write!(f, "could not unmount device ({})", no)
                }
            }
        }
    }
}

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// USB device vendor id
    #[clap(short, long)]
    vendorid: String,

    /// USB device product id
    #[clap(short, long)]
    productid: String,

    /// Input file
    #[clap(short, long)]
    input: String,

    /// Number of bytes to skip before writing
    #[clap(short, long, default_value_t = 131072)]
    bskip: u64,
}

#[derive(Debug)]
struct MountEntry {
    fsname: String,
    dir: String,
    typ: String,
    options: String,
    freq: i32,
    passno: i32,
}

impl MountEntry {
    fn new(
        fsname: String,
        dir: String,
        typ: String,
        options: String,
        freq: i32,
        passno: i32,
    ) -> Self {
        Self {
            fsname,
            dir,
            typ,
            options,
            freq,
            passno,
        }
    }
}

impl DeviceInfo {
    fn new(id_vendor: &str, id_product: &str) -> Result<DeviceInfo> {
        let id_vendor_cs = CString::new(id_vendor).map_err(Error::CStringInit)?;
        let id_product_cs = CString::new(id_product).map_err(Error::CStringInit)?;
        let device_info =
            unsafe { get_device_info_by_vp(id_vendor_cs.as_ptr(), id_product_cs.as_ptr()) };

        if device_info.is_null() {
            Err(Error::Block("devnode is null"))
        } else {
            let deref = unsafe { *device_info };

            // free original copy
            unsafe { libc::free(device_info as *mut _ as *mut libc::c_void) };

            Ok(deref)
        }
    }

    fn get_device_node(&self) -> Result<String> {
        if self.block.is_null() {
            return Err(Error::Block("block is null"));
        }

        let devnode = unsafe { udev_device_get_devnode(self.block) };

        if devnode.is_null() {
            return Err(Error::Block("devnode is null"));
        }

        Ok(cchar_to_string(devnode as *const c_char))
    }

    fn drop(&mut self) {
        if !self.block.is_null() {
            unsafe {
                udev_device_unref(self.block);
            };
        }

        if !self.scsi.is_null() {
            unsafe {
                udev_device_unref(self.scsi);
            };
        }

        if !self.udev.is_null() {
            unsafe {
                udev_unref(self.udev);
            };
        }
    }
}

fn errno() -> i32 {
    if let Some(number) = std::io::Error::last_os_error().raw_os_error() {
        number
    } else {
        0
    }
}

fn get_user_input(text: &str) -> String {
    let mut ret = String::new();

    print!("{}", text);
    let _ = std::io::stdout().flush();
    std::io::stdin()
        .read_line(&mut ret)
        .expect("Error while trying to read from stdin");

    ret
}

fn yesno_prompt(text: &str) -> bool {
    let user_response = loop {
        let resp = get_user_input(text);
        if resp == "y\n" || resp == "n\n" {
            break resp;
        }
    };

    user_response == "y\n"
}

fn cchar_to_string(char_ptr: *const c_char) -> String {
    if !char_ptr.is_null() {
        let cstr = unsafe { CStr::from_ptr(char_ptr) };
        return cstr.to_string_lossy().into_owned();
    } else {
        String::from("")
    }
}

unsafe fn get_mounted_devices_list() -> Result<Vec<MountEntry>> {
    let mut device_list: Vec<MountEntry> = Vec::new();

    let mtab = CString::new("/etc/mtab").map_err(Error::CStringInit)?;
    let mode = CString::new("r").map_err(Error::CStringInit)?;

    let file = setmntent(mtab.as_ptr(), mode.as_ptr());
    if file.is_null() {
        return Ok(device_list);
    }

    loop {
        let entry = getmntent(file);
        if entry.is_null() {
            break;
        } else {
            let fsname = cchar_to_string((*entry).mnt_fsname);
            let dir = cchar_to_string((*entry).mnt_dir);
            let typ = cchar_to_string((*entry).mnt_type);
            let options = cchar_to_string((*entry).mnt_opts);
            let freq = (*entry).mnt_freq;
            let passno = (*entry).mnt_passno;

            let mount_entry = MountEntry::new(fsname, dir, typ, options, freq, passno);
            device_list.push(mount_entry);
        }
    }

    endmntent(file);

    Ok(device_list)
}

fn device_unmount(path: &str) -> Result<()> {
    let path_cs = CString::new(path).map_err(Error::CStringInit)?;
    let result = unsafe { umount(path_cs.as_ptr()) };

    if result == 0 {
        Ok(())
    } else {
        Err(Error::Unmount(errno()))
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    let vendorid = args.vendorid;
    let productid = args.productid;

    let mut device_info =
        DeviceInfo::new(&vendorid, &productid).map_err(|_| Error::DeviceInfoInit)?;

    let devnode = match device_info.get_device_node() {
        Ok(node) => node,
        Err(err) => {
            panic!("Error getting devnode: {:?}", err)
        }
    };

    let device_list = unsafe { get_mounted_devices_list()? };

    let mut can_write = true;
    for device in device_list {
        if device.fsname.starts_with(&devnode) {
            println!("Device {} is mounted at {}", device.fsname, device.dir);

            let user_response = yesno_prompt("Unmount the device (y/n)? ");
            if user_response {
                println!("Trying to unmount...");

                match device_unmount(&device.dir) {
                    Ok(_) => println!("Unmounted successfully!"),
                    Err(e) => {
                        can_write = false;
                        println!("Unmount unsuccessful: {}", e);
                    }
                }
            }

            break;
        }
    }

    // write to device block
    if can_write {
        let prompt_text = format!("Are you sure you want to write to {} (y/n)? ", devnode);
        let user_response = yesno_prompt(&prompt_text);
        if user_response {
            let mut input_file_contents = Vec::new();
            let mut input_file = File::open(args.input).map_err(Error::FileOpen)?;
            input_file
                .read_to_end(&mut input_file_contents)
                .map_err(Error::FileRead)?;

            let device_file = File::options()
                .read(true)
                .write(true)
                .open(&devnode)
                .map_err(Error::FileOpen)?;
            let mut bufwriter = BufWriter::new(&device_file);
            bufwriter
                .seek(SeekFrom::Start(args.bskip))
                .map_err(|_| Error::Seek(args.bskip))?;
            bufwriter
                .write_all(&input_file_contents)
                .map_err(Error::FileWrite)?;
        }
    }

    device_info.drop();

    Ok(())
}
