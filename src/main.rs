use hid_scanner::Scanner;
use passport_rpi::api;
use passport_rpi::rpi::{toggle_light, LEDState};
use rppal::system::DeviceInfo;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use rodio::{Decoder, OutputStream,OutputStreamHandle, source::Source};

fn play_audio(stream_handle: &OutputStreamHandle,path: &str) -> Result<(), Box<dyn Error>> {
    
    let audio = BufReader::new(File::open(path)?);
    stream_handle.play_raw(Decoder::new(audio)?.convert_samples())?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // init
    println!("Running on a {}.", DeviceInfo::new()?.model());
    toggle_light(LEDState::GREEN).unwrap_or(());
    let api = hidapi_rusb::HidApi::new()?;

    // init audio
    println!("Initial audio");
    let (_stream, stream_handle) = OutputStream::try_default()?;
    let panic_audio = "panic.wav";
    let ok_audio = "ok.wav";
    
    // Get all devices
    for device in api.device_list() {
        match device.product_string() {
            Some(product_string) => println!("{}", product_string),
            None => println!("None"),
        }
        println!("{}", device.path().to_string_lossy());
        println!("{:#?}", device);
        match Scanner::open_device(&api, device) {
            Ok(scanner) => {
                println!("Connection Ok");
                match scanner.control(8, 1, 5, 5) {
                    Ok(result) => {
                        println!("Control Status: {:?}", result);
                        if result {
                            println!("Open Device Success!");
                            play_audio(&stream_handle, ok_audio.clone())?;
                            // toggle_light(LEDState::GREEN).unwrap_or(());
                            loop {
                                match scanner.read()? {
                                    Some(rev) => {
                                        let qr = String::from_utf8_lossy(&rev.data);
                                        if api::check_qr(&qr).unwrap_or(false) {
                                            // scanner.control(8, 1, 2, 1).unwrap_or(());
                                            toggle_light(LEDState::GREEN).unwrap_or(());
                                            play_audio(&stream_handle, ok_audio.clone()).unwrap_or(());
                                        } else {
                                            // scanner.control(8, 2, 4, 2).unwrap_or(());
                                            toggle_light(LEDState::RED).unwrap_or(());
                                            play_audio(&stream_handle, panic_audio.clone()).unwrap_or(());
                                        }
                                    },
                                    None => {},
                                }
                            }
                        }
                    },
                    Err(_) => println!("Control Error"),
                }
            },
            Err(_) => println!("Connection Died"),
        }
    }
    Ok(())
}

