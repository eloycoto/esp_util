# Esp utils

ESP utils are a bunch of helper functions to work with esp32-based boards and
make an easy-to-consume interface for common functionalities.


## Cargo example:

```toml
esp_utils = { git = "https://github.com/eloycoto/esp_utils", package = "esp_utils", rev = "9c44e67e231ab9105140671eef64bd5013ef7638"}
```

## Connect to wifi:

```rust
    use esp_utils::wifi::WifiHandler
    let mut socket_set_entries: [SocketStorage; 3] = Default::default();
    let mut wifi_handler = WifiHandler::new_with_sockets(
        init,
        peripherals.WIFI,
        &mut socket_set_entries,
        delay,
    );
    wifi_handler.set_user_pass("ssid_name", "mystrongPassword").unwrap();
    match wifi_handler.start_connection() {
        Err(e) => println!("Cannot start full connection: {:?}", e),
        Ok(()) => {}
    }
```
