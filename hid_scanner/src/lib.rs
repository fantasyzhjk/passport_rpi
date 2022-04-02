use hidapi_rusb::{HidDevice, HidApi, HidError, DeviceInfo};
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use std::io::Cursor;

pub enum State {
    Scan,
    LED,
    ID,
    ScanData,
    Control // LED与蜂鸣器
    
}

pub struct SocketData {
    pub magic: [u8; 2],
    pub state: State,
    pub data: Vec<u8>
}

pub fn calc_xor(buf: Vec<u8>) -> u8 {
    let mut tmp: u8 = 0;
    for i in buf.iter() {
        tmp ^= i
    }
    tmp
}

impl SocketData {
    pub fn new(state: State, data: &[u8]) -> SocketData {
        SocketData {
            magic: [0x55, 0xAA],
            state: state,
            data: data.to_vec()
        }
    }

    pub fn parse(raw_data: &[u8]) -> Option<SocketData> {
        if raw_data[0] == 0x55 && raw_data[1] == 0xAA && raw_data[3] == 0x00 {
            let mut rdr = Cursor::new(raw_data);
            rdr.set_position(2);
            let state = match rdr.read_u8().unwrap() {
                0x22 => State::LED,
                0x24 => State::Scan,
                0x30 => State::ScanData,
                0x02 => State::ID,
                0x04 => State::Control,
                _ => {
                    print!("UnknownState Read: ");
                    raw_data.iter().for_each(|v| {
                        print!("{:02x} ", v);
                    });
                    println!();
                    return None
                },
            };
            rdr.set_position(4);
            let data_length = rdr.read_u16::<LittleEndian>().unwrap();
            Some(SocketData {
                magic: [0x55, 0xAA],
                state: state,
                data: raw_data[6..(data_length + 6) as usize].to_vec()
            })
        } else {
            return None;
        }
    }

    pub fn build(&mut self) -> Vec<u8> {
        let mut tmp: Vec<u8> = Vec::new();
        tmp.push(0x00);
        tmp.append(&mut self.magic.to_vec());
        let state: u8 = match self.state {
            State::Scan => 0x22,
            State::LED => 0x24,
            State::ID => 0x02,
            State::Control => 0x04,
            _ => unreachable!(),
        };
        tmp.push(state);
        tmp.write_u16::<LittleEndian>(self.data.len() as u16).unwrap();
        tmp.append(&mut self.data);
        tmp.push(calc_xor(tmp.clone()));
        return tmp;
    }
}

pub struct Scanner {
    pub device: HidDevice
}

impl Scanner {
    pub fn open(api: &HidApi, vid: u16, ipd: u16) -> Result<Scanner, HidError> {
        let device = api.open(vid, ipd)?;
        Ok(Scanner { device: device })
    }

    pub fn open_device(api: &HidApi, device: &DeviceInfo) -> Result<Scanner, HidError> {
        let device = device.open_device(api)?;
        Ok(Scanner { device: device })
    }

    pub fn control(&self, switch: u8, times: u8, duration: u8, interval: u8) -> Result<bool, HidError> {
        // switch 0x08 为蜂鸣器
        // interval duration 单位为50ms
        let buf  = SocketData::new(State::Control, &[switch, times, interval, duration, 0x00]).build();
        self.device.write(&buf)?;
        match self.read_timeout(1000).unwrap_or(None) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    pub fn light(&self, status: bool) -> Result<bool, HidError> {
        if status {
            let buf  = SocketData::new(State::LED, &[0x01]).build();
            self.device.write(&buf)?;
        } else {
            let buf  = SocketData::new(State::LED, &[0x00]).build();
            let _ = self.device.write(&buf)?;
        }
        match self.read_timeout(1000).unwrap_or(None) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    pub fn scan(&self, status: bool) -> Result<bool, HidError> {
        // 55 aa 05 01 00 01 fa 关闭扫码
        // 55 aa 05 01 00 00 fb 打开扫码
        if status {
            let buf  = SocketData::new(State::Scan, &[0x00]).build();
            self.device.write(&buf)?;
        } else {
            let buf  = SocketData::new(State::Scan, &[0x01]).build();
            let _ = self.device.write(&buf)?;
        }
        match self.read_timeout(1000).unwrap_or(None) {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }

    pub fn read_timeout(&self, timeout: i32) -> Result<Option<SocketData>, HidError> {
        let mut buf = [0u8; 1024];
        let res = self.device.read_timeout(&mut buf[..], timeout)?;
        Ok(SocketData::parse(&buf[..res]))
    }

    pub fn read(&self) -> Result<Option<SocketData>, HidError> {
        let mut buf = [0u8; 1024];
        let res = self.device.read(&mut buf[..])?;
        Ok(SocketData::parse(&buf[..res]))
    }
}