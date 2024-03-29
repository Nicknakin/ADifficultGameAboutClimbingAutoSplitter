#![no_std]

use asr::timer::TimerState;
use asr::{future::next_tick, Address, Process};

asr::async_main!(stable);
asr::panic_handler!();

#[derive(Default, Clone)]
struct State {
    left_hand_grabbed_surface: u64,
    right_hand_grabbed_surface: u64,
    position_x: f32,
    position_y: f32,
    input_listening: u8,
    zone: u8,
}

impl State {
    fn generate(process: &Process, base_address: Address, zone: u8) -> Result<State, asr::Error> {
        const LEFT_HAND_GRABBED_SURFACE_PATH: &[u64] = &[0x7280F8, 0xA0, 0xA98];
        const RIGHT_HAND_GRABBED_SURFACE_PATH: &[u64] = &[0x7280F8, 0xA0, 0xBD8];
        const INPUT_LISTENING_PATH: &[u64] = &[0x7280F8, 0xA0, 0xAF4];
        const POSITION_X_PATH: &[u64] = &[0x7280F8, 0xA0, 0xA88, 0x30, 0x10, 0xE0];
        const POSITION_Y_PATH: &[u64] = &[0x7280F8, 0xA0, 0xA88, 0x30, 0x10, 0xE4];

        let left_hand_grabbed_surface = process.read_pointer_path::<u64>(
            base_address,
            asr::PointerSize::Bit64,
            LEFT_HAND_GRABBED_SURFACE_PATH,
        )?;

        let right_hand_grabbed_surface = process.read_pointer_path::<u64>(
            base_address,
            asr::PointerSize::Bit64,
            RIGHT_HAND_GRABBED_SURFACE_PATH,
        )?;

        let input_listening = process.read_pointer_path::<u8>(
            base_address,
            asr::PointerSize::Bit64,
            INPUT_LISTENING_PATH,
        )?;

        let position_x = process.read_pointer_path::<f32>(
            base_address,
            asr::PointerSize::Bit64,
            POSITION_X_PATH,
        )?;

        let position_y = process.read_pointer_path::<f32>(
            base_address,
            asr::PointerSize::Bit64,
            POSITION_Y_PATH,
        )?;

        return Ok(State {
            left_hand_grabbed_surface,
            right_hand_grabbed_surface,
            position_x,
            position_y,
            input_listening,
            zone,
        });
    }

    fn log(&self) {
        asr::print_limited::<1024>(&format_args!(
            "{:?} - ({:.2?}, {:.2?}) - ({:x?}, {:x?}) - Zone:  {:?}",
            self.input_listening,
            self.position_x,
            self.position_y,
            self.left_hand_grabbed_surface,
            self.right_hand_grabbed_surface,
            self.zone,
        ));
    }

    fn should_start(&self, old_state: &State) -> bool {
        ((old_state.left_hand_grabbed_surface == 0 && self.left_hand_grabbed_surface != 0)
            || (old_state.right_hand_grabbed_surface == 0 && self.right_hand_grabbed_surface != 0))
            && (self.position_y < 2f32)
    }

    fn should_split(&mut self) -> bool {
        const MOUNTAIN: u8 = 0b00000001;
        const JUNGLE: u8 = 0b00000010;
        const FACTORY: u8 = 0b00000100;
        const POOL: u8 = 0b00001000;
        const CONSTRUCTION: u8 = 0b00010000;
        const CAVE: u8 = 0b00100000;
        const ICE: u8 = 0b01000000;
        const CREDITS: u8 = 0b10000000;
        if (self.zone & MOUNTAIN == 0) && self.position_y > 31f32 {
            self.zone = self.zone | MOUNTAIN;
            return true;
        }
        if (self.zone & JUNGLE == 0) && self.position_y > 55f32 && self.position_x < 0f32 {
            self.zone = self.zone | JUNGLE;
            return true;
        }
        if (self.zone & FACTORY == 0)
            && self.position_y > 80f32
            && self.position_y < 87f32
            && self.position_x > 8f32
        {
            self.zone = self.zone | FACTORY;
            return true;
        }
        if (self.zone & POOL == 0) && self.position_y > 109f32 && self.position_x < 20f32 {
            self.zone = self.zone | POOL;
            return true;
        }
        if (self.zone & CONSTRUCTION == 0) && self.position_y > 135f32 {
            self.zone = self.zone | CONSTRUCTION;
            return true;
        }
        if (self.zone & CAVE == 0) && self.position_y > 152f32 {
            self.zone = self.zone | CAVE;
            return true;
        }
        if (self.zone & ICE == 0) && self.position_y > 204f32 && self.position_x < 47f32 {
            self.zone = self.zone | ICE;
            return true;
        }
        if (self.zone & CREDITS == 0) && self.position_y > 245f32 {
            self.zone = self.zone | CREDITS;
            return true;
        }

        return false;
    }

    fn should_reset(&self, old_state: &State) -> bool {
        (old_state.input_listening & 1) != 1 && (self.input_listening & 1) == 0
    }
}

async fn main() {
    loop {
        let process = Process::wait_attach("A Difficult Game About Climbing.exe").await;
        process
            .until_closes(async {
                asr::print_message("Intializing References...");
                // Load initial locations for libraries from process
                let Ok(mono_address) = process.get_module_address("mono-2.0-bdwgc.dll") else {
                    return;
                };

                #[cfg(debug_assertions)]
                asr::print_limited::<18>(&format_args!("0x{:x?}", mono_address));

                let mut current_state = State::default();

                loop {
                    //Load each of the relevant values from memory in to variables
                    let timer_state = &asr::timer::state();
                    let old_state = current_state.clone();
                    current_state =
                        match State::generate(&process, mono_address, current_state.zone) {
                            Ok(state) => state,
                            Err(_) => {
                                continue;
                            }
                        };

                    #[cfg(debug_assertions)]
                    current_state.log();

                    let unstarted_states = [
                        TimerState::NotRunning,
                        TimerState::Ended,
                        TimerState::Unknown,
                    ];

                    if unstarted_states.contains(timer_state)
                        && current_state.should_start(&old_state)
                    {
                        #[cfg(debug_assertions)]
                        asr::print_message("Starting run!");
                        asr::timer::start();
                    }

                    if *timer_state == TimerState::Running && current_state.should_split() {
                        if cfg!(debug_assertions) {
                            asr::print_limited::<1024>(&format_args!(
                                "Splitting! {:?}",
                                current_state.zone
                            ))
                        }
                        asr::timer::split();
                    }

                    if *timer_state != TimerState::NotRunning
                        && current_state.should_reset(&old_state)
                    {
                        #[cfg(debug_assertions)]
                        asr::print_message("Reseting Run");

                        current_state.zone = 0;
                        asr::timer::reset();
                    }

                    if *timer_state == TimerState::NotRunning {
                        current_state.zone = 0;
                    }

                    next_tick().await;
                }
            })
            .await;
    }
}
