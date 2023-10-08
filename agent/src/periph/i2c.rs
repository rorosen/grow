use nix::libc;
use std::os::fd::AsRawFd;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::Error;

const I2C_DEVICE_PATH: &str = "/dev/i2c-1";
const REQ_SLAVE: libc::c_ulong = 0x0706;

pub struct I2C {
    dev: File,
}

impl I2C {
    pub async fn new(slave_address: u8) -> Result<Self, Error> {
        let dev = OpenOptions::new()
            .read(true)
            .write(true)
            .open(I2C_DEVICE_PATH)
            .await
            .map_err(|err| Error::OpenError {
                file: I2C_DEVICE_PATH,
                err,
            })?;

        if unsafe {
            libc::ioctl(
                dev.as_raw_fd(),
                REQ_SLAVE,
                libc::c_ulong::from(slave_address),
            )
        } == -1
        {
            return Err(Error::I2cSlaveAddrError);
        }

        Ok(Self { dev })
    }

    pub async fn read_byte(&mut self, address: u8) -> Result<u8, Error> {
        self.dev
            .write_all(&[address])
            .await
            .map_err(Error::I2cWriteError)?;

        self.dev.read_u8().await.map_err(Error::I2cReadError)
    }

    pub async fn write_byte(&mut self, address: u8, data: u8) -> Result<(), Error> {
        self.dev
            .write_all(&[address, data])
            .await
            .map_err(Error::I2cWriteError)
    }

    pub async fn set_bits(&mut self, address: u8, mask: u8) -> Result<(), Error> {
        let data = self.read_byte(address).await?;

        self.write_byte(address, data | mask).await
    }

    pub async fn read_u16(&mut self, address: u8) -> Result<u16, Error> {
        self.dev
            .write_all(&[address])
            .await
            .map_err(Error::I2cWriteError)?;
        self.dev.read_u16().await.map_err(Error::I2cReadError)
    }
}
