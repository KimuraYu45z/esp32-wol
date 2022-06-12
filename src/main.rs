#![no_main]

use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported

mod config;

use anyhow::bail;
use config::*;
use embedded_svc::http::client::*;
use embedded_svc::io::Bytes;
use embedded_svc::ipv4;
use embedded_svc::ping::Ping;
use embedded_svc::wifi::{Configuration, *};
use esp_idf_svc::http::client::*;
use esp_idf_svc::netif::*;
use esp_idf_svc::nvs::*;
use esp_idf_svc::ping;
use esp_idf_svc::sysloop::*;
use esp_idf_svc::wifi::*;
use log::*;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use wake_on_lan::MagicPacket;

fn ping(ip_settings: &ipv4::ClientSettings) -> Result<(), anyhow::Error> {
    info!("About to do some pings for {:?}", ip_settings);

    let ping_summary =
        ping::EspPing::default().ping(ip_settings.subnet.gateway, &Default::default())?;
    if ping_summary.transmitted != ping_summary.received {
        bail!(
            "Pinging gateway {} resulted in timeouts",
            ip_settings.subnet.gateway
        );
    }

    info!("Pinging done");

    Ok(())
}

fn init_wifi(
    netif_stack: Arc<EspNetifStack>,
    sys_loop_stack: Arc<EspSysLoopStack>,
    default_nvs: Arc<EspDefaultNvs>,
) -> Result<Box<EspWifi>, anyhow::Error> {
    let mut wifi = Box::new(EspWifi::new(netif_stack, sys_loop_stack, default_nvs)?);

    info!("Wifi created, about to scan");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == WIFI_SSID);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            WIFI_SSID, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            WIFI_SSID
        );
        None
    };

    wifi.set_configuration(&Configuration::Mixed(
        ClientConfiguration {
            ssid: WIFI_SSID.into(),
            password: WIFI_PASS.into(),
            channel,
            ..Default::default()
        },
        AccessPointConfiguration {
            ssid: "aptest".into(),
            channel: channel.unwrap_or(1),
            ..Default::default()
        },
    ))?;

    info!("Wifi configuration set, about to get status");

    wifi.wait_status_with_timeout(Duration::from_secs(20), |status| !status.is_transitional())
        .map_err(|e| anyhow::anyhow!("Unexpected Wifi status: {:?}", e))?;

    let status = wifi.get_status();

    if let Status(
        ClientStatus::Started(ClientConnectionStatus::Connected(ClientIpStatus::Done(ip_settings))),
        ApStatus::Started(ApIpStatus::Done),
    ) = status
    {
        info!("Wifi connected");

        ping(&ip_settings)?;
    } else {
        bail!("Unexpected Wifi status: {:?}", status);
    }

    Ok(wifi)
}

fn api_get() -> Result<bool, anyhow::Error> {
    info!("About to get content from {}", API_URL_GET);

    let mut client = EspHttpClient::new(&EspHttpClientConfiguration {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),

        ..Default::default()
    })?;

    let response = client.get(API_URL_GET)?.submit()?;

    let body: Result<Vec<u8>, _> = Bytes::<_, 64>::new(response.reader()).take(1024).collect();
    let body = body?;
    let body = String::from_utf8_lossy(&body).into_owned();

    let result = body.parse::<bool>().unwrap_or(false);

    Ok(result)
}

fn api_post() -> Result<bool, anyhow::Error> {
    info!("About to post content to {}", API_URL_POST);

    let mut client = EspHttpClient::new(&EspHttpClientConfiguration {
        crt_bundle_attach: Some(esp_idf_sys::esp_crt_bundle_attach),

        ..Default::default()
    })?;

    let response = client.post(API_URL_POST)?.submit()?;

    let body: Result<Vec<u8>, _> = Bytes::<_, 64>::new(response.reader()).take(1024).collect();
    let body = body?;
    let body = String::from_utf8_lossy(&body).into_owned();

    let result = body.parse::<bool>().unwrap_or(false);

    Ok(result)
}

fn patrol_iteration() -> Result<(), anyhow::Error> {
    let switch = api_get()?;

    if switch {
        info!("Wake on LAN to MAC address: {:X?}", WOL_MAC_ADDRESS);

        let magic_packet = MagicPacket::new(&WOL_MAC_ADDRESS);
        magic_packet.send()?;

        api_post()?;
    }

    Ok(())
}

#[no_mangle]
fn app_main() -> Result<(), anyhow::Error> {
    // Temporary. Will disappear once ESP-IDF 4.4 is released, but for now it is necessary to call this function once,
    // or else some patches to the runtime implemented by esp-idf-sys might not link properly.
    esp_idf_sys::link_patches();

    println!("Hello, world!");

    let netif_stack = Arc::new(EspNetifStack::new()?);
    let sys_loop_stack = Arc::new(EspSysLoopStack::new()?);
    let default_nvs = Arc::new(EspDefaultNvs::new()?);
    let wifi = init_wifi(netif_stack, sys_loop_stack, default_nvs)?;

    while patrol_iteration().is_ok() {
        thread::sleep(Duration::from_millis(INTERVAL_MILLIS))
    }

    for i in 0..3 {
        println!("Restarting in {} seconds...", 3 - i);
        thread::sleep(Duration::from_secs(1));
    }

    drop(wifi);
    println!("Wifi stopped");

    unsafe {
        esp_idf_sys::esp_restart();
    }

    Ok(())
}
