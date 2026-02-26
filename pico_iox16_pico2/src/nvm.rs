use core::{
    ptr::addr_of,
    sync::atomic::{AtomicBool, Ordering},
};

use cortex_m::interrupt;
use pico_iox16_firmware::nvm::{NonvolatileStorage, default_nonvolatile_data};
use rp235x_hal::rom_data::{flash_range_erase, flash_range_program};

use crate::runtime::Board;

#[unsafe(link_section = ".config")]
#[used]
static mut CONFIG: [u8; 4096] = default_nonvolatile_data();

static CONFIG_LOCK: AtomicBool = AtomicBool::new(false);

pub struct Nvm(());
impl Drop for Nvm {
    fn drop(&mut self) {
        CONFIG_LOCK.store(false, Ordering::Release);
    }
}
impl Nvm {
    pub fn take() -> Option<Self> {
        if CONFIG_LOCK
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(Self(()))
        } else {
            None
        }
    }
}

impl NonvolatileStorage<Board> for Nvm {
    type Error = core::convert::Infallible;

    fn read(&self) -> nb::Result<[u8; 4096], Self::Error> {
        Ok(unsafe { addr_of!(CONFIG).read_volatile() })
    }

    fn write(&self, data: &[u8; 4096]) -> nb::Result<(), Self::Error> {
        interrupt::free(|_| unsafe {
            flash_range_erase(addr_of!(CONFIG) as u32, 4096, 4096, 0xD8);
            flash_range_program(
                addr_of!(CONFIG) as u32,
                data.as_ptr(),
                4096,
            );
        });
        Ok(())
    }
}
