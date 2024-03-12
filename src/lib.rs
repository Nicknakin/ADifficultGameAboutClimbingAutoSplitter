#![no_std]

use asr::{future::next_tick, Address, Process};

asr::async_main!(stable);
asr::panic_handler!();

// read_pointer_path
//  get_module_address
/*
      long leftHandGrabbedSurface : "mono-2.0-bdwgc.dll", 0x7280F8, 0xA0, 0xA98;
    long rightHandGrabbedSurface : "mono-2.0-bdwgc.dll", 0x7280F8, 0xA0, 0xBD8;

    bool listenToInput : "mono-2.0-bdwgc.dll", 0x7280F8, 0xA0, 0xAF4;
    float positionX : "UnityPlayer.dll", 0x1B2ACB0, 0x20, 0x5E0, 0x28, 0x270, 0xC8, 0x4C, 0x20, 0x10, 0x20;
    float positionY : "UnityPlayer.dll", 0x1B2ACB0, 0x20, 0x5E0, 0x28, 0x270, 0xC8, 0x4C, 0x20, 0x10, 0x24;
*/

async fn main() {
    // TODO: Set up some general state and settings.
    let left_hand_grabbed_surface_path = [0x7280F8, 0xA0, 0xA98];
    let right_hand_grabbed_surface_path = [0x7280F8, 0xA0, 0xBD8];
    let listen_to_input_path = [0x7280_f8, 0x_a0, 0x_af4];
    let position_x_path = [
        0x1B2ACB0, 0x20, 0x5E0, 0x28, 0x270, 0xC8, 0x4C, 0x20, 0x10, 0x20,
    ];
    let position_y_path = [
        0x1B2ACB0, 0x20, 0x5E0, 0x28, 0x270, 0xC8, 0x4C, 0x20, 0x10, 0x24,
    ];

    loop {
        let process = Process::wait_attach("A Difficult Game About Climbing.exe").await;
        process
            .until_closes(async {
                // Load initial locations for libraries from process
                let mono_address: Address =
                    process.get_module_address("mono-2.0-bdwgc.dll").unwrap();
                let unity_address: Address = process.get_module_address("UnityPlayer.dll").unwrap();

                loop {
                    //Load each of the relevant values from memory in to variables
                    let left_hand_grabbed_surface: u64 = process
                        .read_pointer_path::<u64>(
                            mono_address,
                            asr::PointerSize::Bit64,
                            &left_hand_grabbed_surface_path,
                        )
                        .unwrap();
                    let right_hand_grabbed_surface: u64 = process
                        .read_pointer_path(
                            mono_address,
                            asr::PointerSize::Bit64,
                            &right_hand_grabbed_surface_path,
                        )
                        .unwrap();
                    let listen_to_input: bool = process
                        .read_pointer_path(
                            mono_address,
                            asr::PointerSize::Bit16,
                            &listen_to_input_path,
                        )
                        .unwrap();
                    let position_x: f64 = process
                        .read_pointer_path(unity_address, asr::PointerSize::Bit64, &position_x_path)
                        .unwrap();
                    let position_y: f64 = process
                        .read_pointer_path(unity_address, asr::PointerSize::Bit64, &position_y_path)
                        .unwrap();

                    // TODO: Do something on every tick.
                    next_tick().await;
                }
            })
            .await;
    }
}
