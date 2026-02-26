use std::{cmp::max, time::{Duration, Instant}};

use anyhow::{Context as _, Result};
use pico_iox16_protocol::{Message, RequestTrait, master_next};
use tokio::{io::{AsyncReadExt as _, AsyncWriteExt as _}, time::timeout};
use tokio_serial::{SerialPort, SerialStream};
use zerocopy::{IntoBytes, };

pub struct Protocol {
    device: SerialStream,
    buf_len: usize,
    buf: [u8; size_of::<Message<[u8; 1024]>>()],
}

impl Protocol {
    pub fn new(device: SerialStream) -> Self {
        Self {
            device,
            buf_len: 0,
            buf: [0; size_of::<Message<[u8; 1024]>>()],
        }
    }

    pub fn baudrate(&self) -> u32 {
        self.device.baud_rate().unwrap()
    }

    pub async fn send_request<P: RequestTrait, R>(
        &mut self,
        address: u16,
        payload: P,
        handle_response: impl for<'v> FnOnce(&P::Response) -> Result<R>,
    ) -> Result<R> {
        let timeout = Duration::from_micros(max(P::TIMEOUT_US.into(), 1000));
        let message = Message::new_request(address, P::COMMAND, payload);
        self.device.write_all(message.as_bytes()).await.context(format!("Sending {} request", P::COMMAND))?;
        self.device.flush().await.context(format!("Sending {} request", P::COMMAND))?;
        let start = Instant::now();
        let mut elapsed = Duration::ZERO;
        loop {
            if elapsed >= timeout {
                return Err(anyhow::anyhow!("Timed out waiting for response"));
            }
            let Ok(n) = tokio::time::timeout(timeout - elapsed, self.device.read(&mut self.buf[self.buf_len..])).await else {
                elapsed = start.elapsed();
                continue;
            };
            let n = n.context(format!("Waiting for  {} response", P::COMMAND))?;
            self.buf_len += n;
            let (maybe_message, processed) = master_next(&self.buf[..self.buf_len]);
            if let Some((response_address, response)) = maybe_message {
                if response_address != address {
                    return Err(anyhow::anyhow!("Received response from unexpected address 0x{:02X} (expected 0x{:02X})", response_address, address));
                }
                if let Some(response) = P::get_response(response) {
                    let result = handle_response(response);
                    self.buf_len -= processed;
                    self.buf.copy_within(processed.., 0);
                    return result;
                } else {
                    return Err(anyhow::anyhow!("Received response with unexpected command {:?} (expected {:?})", response.command(), P::COMMAND));
                }
            }
            self.buf_len -= processed;
            self.buf.copy_within(processed.., 0);
            elapsed = start.elapsed();
        }
    }
}
