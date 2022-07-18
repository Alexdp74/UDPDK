#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

use std::ffi::CString;
use std::io::Error;
use std::net::SocketAddr;
use std::net::{IpAddr, Ipv4Addr};
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::os::raw::c_void;
use std::{thread, time};

type in_port_t = u16;
pub type in_addr_t = u32;

pub const SOCK_DGRAM: u32 = __socket_type_SOCK_DGRAM;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct sockaddr_in {
    sin_family: sa_family_t,
    sin_port: in_port_t,
    sin_addr: in_addr,
    sin_zero: [::std::os::raw::c_char; 8usize],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct in_addr {
    pub s_addr: in_addr_t,
}

#[derive(Debug)]
pub struct UDPDK {
    sockfd: i32,
}

impl UDPDK {
    /// Initialize udpdk. Returns 0 if everything is ok, -1 otherwise.
    /// args should contain ["progname", "-c", "path/to/config.ini"], e.g.,:
    /// ```
    ///    use std::env;
    ///    let progname = &env::args().collect::<Vec<String>>()[0];
    ///    let args: Vec<String> = vec![
    ///        progname.clone(),
    ///        "-c".to_string(),
    ///        "../config.ini".to_string(),
    ///    ];
    /// ```
    pub fn init(args: Vec<String>) -> i32 {
        let args: Vec<CString> = args
            .iter()
            .map(|arg| CString::new(&arg[..]).unwrap())
            .collect::<Vec<CString>>();

        // create a vector of zero terminated strings
        // convert the strings to raw pointers
        let mut c_args = args
            .iter()
            .map(|arg| arg.as_ptr())
            .collect::<Vec<*const c_char>>();
        c_args.push(std::ptr::null());

        let retval =
            unsafe { udpdk_init((c_args.len() - 1) as c_int, c_args.as_ptr() as *mut *mut i8) };

        if retval == 0 {
            let ten_sec = time::Duration::from_millis(10000);
            thread::sleep(ten_sec);
        }
        retval
    }

    /// Send a signal to the poller for it to quit
    pub fn interrupt(signum: i32) {
        unsafe {
            udpdk_interrupt(signum);
        }
    }

    /// Cleanup udpdk and dpdk
    pub fn cleanup() {
        unsafe {
            udpdk_cleanup();
        }
    }

    /// Create a new UDPDK socket
    pub fn socket(domain: i32, r#type: u32, protocol: i32) -> Self {
        let sockfd = unsafe { udpdk_socket(domain, r#type.try_into().unwrap(), protocol) };
        UDPDK { sockfd }
    }

    /// Bind UDPDK socket s to address addr
    pub fn bind(&self, addr: SocketAddr) -> i32 {
        assert!(addr.is_ipv4());

        let octets = match addr.ip() {
            IpAddr::V4(ip) => ip.octets(),
            _ => panic!("Not an IPv4!"),
        };

        let udpdk_addr = sockaddr_in {
            sin_family: AF_INET as sa_family_t,
            sin_port: addr.port().to_be(),
            sin_addr: in_addr {
                s_addr: u32::from_ne_bytes(octets),
            },
            sin_zero: [0; 8],
        };

        unsafe {
            udpdk_bind(
                self.sockfd,
                &udpdk_addr as *const _ as *const sockaddr,
                std::mem::size_of::<sockaddr>().try_into().unwrap(),
            )
        }
    }

    /// Send buffer buf to address dest on UDPDK socket sockfd and return the amount of bytes written
    pub fn sendto(&self, buf: &[u8], flags: i32, dest: SocketAddr) -> i64 {
        assert!(dest.is_ipv4());

        let octets = match dest.ip() {
            IpAddr::V4(ip) => ip.octets(),
            _ => panic!("Not an IPv4!"),
        };

        let udpdk_addr = sockaddr_in {
            sin_family: AF_INET as sa_family_t,
            sin_port: dest.port().to_be(),
            sin_addr: in_addr {
                s_addr: u32::from_ne_bytes(octets),
            },
            sin_zero: [0; 8],
        };

        unsafe {
            udpdk_sendto(
                self.sockfd,
                buf.as_ptr() as *const c_void,
                buf.len().try_into().unwrap(),
                flags,
                &udpdk_addr as *const _ as *const sockaddr,
                std::mem::size_of::<sockaddr>().try_into().unwrap(),
            )
        }
    }

    /// Receive data on UDPDK socket sockfd and place it in buf. Returns a Result<(bytes read, sender
    /// SocketAddr)>.
    pub fn recvfrom(&self, buf: &mut [u8], flags: u32) -> Result<(usize, SocketAddr), Error> {
        let mut src_addr = sockaddr_in {
            sin_family: AF_INET as sa_family_t,
            sin_port: 0,
            sin_addr: in_addr { s_addr: 0 },
            sin_zero: [0; 8],
        };
        let mut addr_len: socklen_t = std::mem::size_of::<sockaddr>().try_into().unwrap();

        let retval = unsafe {
            udpdk_recvfrom(
                self.sockfd,
                buf.as_mut_ptr() as *mut c_void,
                buf.len().try_into().unwrap(),
                flags.try_into().unwrap(),
                &mut src_addr as *mut _ as *mut sockaddr,
                &mut addr_len,
            )
        };

        if retval > 0 {
            let sender = SocketAddr::new(
                IpAddr::V4(Ipv4Addr::from(u32::from_be(src_addr.sin_addr.s_addr))),
                u16::from_be(src_addr.sin_port),
            );
            Ok((retval.try_into().unwrap(), sender))
        } else {
            Err(Error::last_os_error())
        }
    }

    /// Close the UDPDK socket s
    fn close(&self) {
        unsafe {
            udpdk_close(self.sockfd);
        }
    }
}

// This trivial implementation of `drop` adds a print to console.
impl Drop for UDPDK {
    fn drop(&mut self) {
        self.close();
    }
}
