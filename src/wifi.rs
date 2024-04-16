use esp_hal::delay::Delay;
use esp_hal::prelude::*;
use esp_println::println;

use esp_wifi::wifi::utils::create_network_interface;
use esp_wifi::wifi::{
    ClientConfiguration, Configuration, WifiController, WifiError, WifiStaDevice,
};
use esp_wifi::wifi_interface::WifiStack;
use esp_wifi::{current_millis, EspWifiInitialization};
use smoltcp::iface::SocketStorage;

#[derive(Debug, Clone)]
pub enum WifiHandlerError {
    #[allow(dead_code)]
    WifiError(WifiError),
    NotStarted,
    CantConnect,
    NoIface,
}

pub struct WifiHandler<'a> {
    wifi_stack: WifiStack<'a, WifiStaDevice>,
    controller: WifiController<'a>,
    poll_limit: u32,
    delay: Delay,
}

impl<'a> WifiHandler<'a> {
    pub fn new_with_sockets(
        init: EspWifiInitialization,
        wifi: impl esp_hal::peripheral::Peripheral<P = esp_hal::peripherals::WIFI> + 'static,
        socket_entries: &'a mut [SocketStorage<'a>; 3],
        delay: Delay,
    ) -> Self {
        let (iface, device, controller, sockets) =
            create_network_interface(&init, wifi, WifiStaDevice, socket_entries).unwrap();

        let wifi_stack = WifiStack::new(iface, device, sockets, current_millis);
        WifiHandler {
            wifi_stack,
            controller,
            poll_limit: 1000000,
            delay,
        }
    }
    pub fn set_user_pass(&mut self, ssid_val: &str, password_val: &str) -> Result<(), WifiError> {
        let ssid: heapless::String<32> = heapless::String::try_from(ssid_val).unwrap();
        let password: heapless::String<64> = heapless::String::try_from(password_val).unwrap();

        self.set_config(ClientConfiguration {
            ssid,
            password,
            ..Default::default()
        })
    }

    pub fn set_config(&mut self, client_config: ClientConfiguration) -> Result<(), WifiError> {
        let client_config = Configuration::Client(client_config);
        self.controller.set_configuration(&client_config)
    }

    fn poll<F, T, E>(limit: u32, mut func: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        E: Clone,
    {
        let mut last_error = None;

        for _ in 0..limit {
            match func() {
                Ok(result) => return Ok(result),
                Err(err) => {
                    last_error = Some(err.clone());
                }
            }
        }
        Err(last_error.unwrap())
    }

    fn parse_wifi_error(e: WifiError) -> WifiHandlerError {
        WifiHandlerError::WifiError(e)
    }

    pub fn start(&mut self) -> Result<(), WifiHandlerError> {
        self.controller.start().map_err(Self::parse_wifi_error)
    }

    pub fn connect(&mut self) -> Result<(), WifiHandlerError> {
        self.controller.connect().map_err(Self::parse_wifi_error)
    }

    pub fn is_connected(&mut self) -> Result<bool, WifiHandlerError> {
        self.controller
            .is_connected()
            .map_err(Self::parse_wifi_error)
    }

    pub fn start_connection(&mut self) -> Result<(), WifiHandlerError> {
        self.start()?;
        if !self
            .controller
            .is_started()
            .map_err(Self::parse_wifi_error)?
        {
            return Err(WifiHandlerError::NotStarted);
        }

        self.connect()?;

        Self::poll(self.poll_limit, || {
            let res = self.is_connected()?;
            if res {
                return Ok(());
            }
            self.delay.delay_ms(100u32);
            Err(WifiHandlerError::CantConnect)
        })?;

        println!("{:?}", self.controller.is_connected());

        println!("Wait to get an ip address");

        Self::poll(self.poll_limit, || {
            self.wifi_stack.work();

            if self.wifi_stack.is_iface_up() {
                return Ok(());
            }
            self.delay.delay_ms(100u32);
            return Err(WifiHandlerError::NoIface);
        })?;

        println!("got ip {:?}", self.wifi_stack.get_ip_info());

        Ok(())
    }

    pub fn get_socket<'s>(
        &'s self,
        rx_buffer: &'a mut [u8],
        tx_buffer: &'a mut [u8],
    ) -> esp_wifi::wifi_interface::Socket<'s, 'a, WifiStaDevice>
    where
        's: 'a,
    {
        return self.wifi_stack.get_socket(rx_buffer, tx_buffer);
    }
}
