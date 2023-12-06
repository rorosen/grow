use nix::libc;
use std::os::fd::AsRawFd;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::error::AppError;

const I2C_DEVICE_PATH: &str = "/dev/i2c-1";
const REQ_SLAVE: libc::c_ulong = 0x0706;

pub struct I2C {
    dev: File,
}

impl I2C {
    pub async fn new(slave_address: u8) -> Result<Self, AppError> {
        let dev = OpenOptions::new()
            .read(true)
            .write(true)
            .open(I2C_DEVICE_PATH)
            .await
            .map_err(|err| AppError::I2cOpenError {
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
            return Err(AppError::I2cSlaveAddrError);
        }

        Ok(Self { dev })
    }

    pub async fn read_reg_byte(&mut self, address: u8) -> Result<u8, AppError> {
        self.dev
            .write_u8(address)
            .await
            .map_err(AppError::I2cWriteError)?;

        self.dev.read_u8().await.map_err(AppError::I2cReadError)
    }

    pub async fn write_reg_byte(&mut self, address: u8, data: u8) -> Result<(), AppError> {
        self.dev
            .write_all(&[address, data])
            .await
            .map_err(AppError::I2cWriteError)
    }

    pub async fn set_reg_bits(&mut self, address: u8, mask: u8) -> Result<(), AppError> {
        let data = self.read_reg_byte(address).await?;

        self.write_reg_byte(address, data | mask).await
    }

    pub async fn read_reg_u16(&mut self, address: u8) -> Result<u16, AppError> {
        self.dev
            .write_u8(address)
            .await
            .map_err(AppError::I2cWriteError)?;
        self.dev.read_u16().await.map_err(AppError::I2cReadError)
    }

    pub async fn write_reg_u16(&mut self, address: u8, data: u16) -> Result<(), AppError> {
        let data = data.to_be_bytes();
        self.dev
            .write_all(&[address, data[0], data[1]])
            .await
            .map_err(AppError::I2cWriteError)
    }

    pub async fn read_reg_bytes(&mut self, address: u8, buf: &mut [u8]) -> Result<usize, AppError> {
        self.dev
            .write_u8(address)
            .await
            .map_err(AppError::I2cWriteError)?;
        self.dev.read(buf).await.map_err(AppError::I2cReadError)
    }

    pub async fn read_bytes(&mut self, buf: &mut [u8]) -> Result<usize, AppError> {
        self.dev.read(buf).await.map_err(AppError::I2cReadError)
    }

    pub async fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), AppError> {
        self.dev
            .write_all(bytes)
            .await
            .map_err(AppError::I2cWriteError)
    }
}
