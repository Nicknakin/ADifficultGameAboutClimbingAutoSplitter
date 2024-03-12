#![no_std]

use asr::timer::TimerState;
use asr::{future::next_tick, Address, Process};

asr::async_main!(stable);
asr::panic_handler!();

#[derive(Clone)]
enum Zone {
    Mountain,
    Jungle,
    Gears,
    Pool,
    Construction,
    Cave,
    Ice,
    Credits,
}
impl Default for Zone {
    fn default() -> Self {
        Zone::Mountain
    }
}

impl Zone {
    fn split(&self) -> Zone {
        match &self {
            Zone::Mountain => Zone::Jungle,
            Zone::Jungle => Zone::Gears,
            Zone::Gears => Zone::Pool,
            Zone::Pool => Zone::Construction,
            Zone::Construction => Zone::Cave,
            Zone::Cave => Zone::Ice,
            Zone::Ice => Zone::Credits,
            Zone::Credits => Zone::Mountain,
        }
    }
    fn to_string(&self) -> &str {
        match &self {
            Zone::Mountain => "Mountain",
            Zone::Jungle => "Jungle",
            Zone::Gears => "Gears",
            Zone::Pool => "Pool",
            Zone::Construction => "Construction",
            Zone::Cave => "Cave",
            Zone::Ice => "Ice",
            Zone::Credits => "Credits",
        }
    }
}

#[derive(Default, Clone)]
struct State {
    left_hand_grabbed_surface: u64,
    right_hand_grabbed_surface: u64,
    position_x: f32,
    position_y: f32,
    zone: Zone,
}

impl State {
    fn update(&mut self, process: &Process, base_address: Address) {
        const LEFT_HAND_GRABBED_SURFACE_PATH: &[u64] = &[0x7280F8, 0xA0, 0xA98];
        const RIGHT_HAND_GRABBED_SURFACE_PATH: &[u64] = &[0x7280F8, 0xA0, 0xBD8];
        const POSITION_X_PATH: &[u64] = &[0x7280F8, 0xA0, 0xA88, 0x30, 0x10, 0xE0];
        const POSITION_Y_PATH: &[u64] = &[0x7280F8, 0xA0, 0xA88, 0x30, 0x10, 0xE4];

        self.left_hand_grabbed_surface = process
            .read_pointer_path::<u64>(
                base_address,
                asr::PointerSize::Bit64,
                LEFT_HAND_GRABBED_SURFACE_PATH,
            )
            .unwrap_or(0);

        self.right_hand_grabbed_surface = process
            .read_pointer_path(
                base_address,
                asr::PointerSize::Bit64,
                RIGHT_HAND_GRABBED_SURFACE_PATH,
            )
            .unwrap_or(0);

        self.position_x = process
            .read_pointer_path(base_address, asr::PointerSize::Bit64, POSITION_X_PATH)
            .unwrap_or(f32::NAN);

        self.position_y = process
            .read_pointer_path(base_address, asr::PointerSize::Bit64, POSITION_Y_PATH)
            .unwrap_or(f32::NAN);
    }

    fn log(&self) {
        asr::print_limited::<1024>(&format_args!(
            "({:.2?}, {:.2?}) - ({:x?}, {:x?}) - Zone:  {:?}",
            self.position_x,
            self.position_y,
            self.left_hand_grabbed_surface,
            self.right_hand_grabbed_surface,
            self.zone.to_string(),
        ));
    }

    fn should_start(&self, old_state: &State) -> bool {
        ((old_state.left_hand_grabbed_surface == 0 && self.left_hand_grabbed_surface != 0)
            || (old_state.right_hand_grabbed_surface == 0 && self.right_hand_grabbed_surface != 0))
            && (self.position_y < 2f32)
    }

    fn should_split(&mut self) -> bool {
        let should_split = match self.zone {
            Zone::Mountain => self.position_y > 31f32,
            Zone::Jungle => self.position_y > 55f32 && self.position_x < 0f32,
            Zone::Gears => {
                self.position_y > 80f32 && self.position_y < 87f32 && self.position_x > 8f32
            }
            Zone::Pool => self.position_y > 109f32 && self.position_x < 20f32,
            Zone::Construction => self.position_y > 135f32,
            Zone::Cave => self.position_y > 152f32,
            Zone::Ice => self.position_y > 204f32 && self.position_x < 47f32,
            Zone::Credits => self.position_y > 245f32,
        };
        if should_split {
            self.zone = self.zone.split();
        }
        should_split
    }
}

async fn main() {
    loop {
        let process = Process::wait_attach("A Difficult Gam").await;
        process
            .until_closes(async {
                asr::print_message("Intializing References...");
                // Load initial locations for libraries from process
                let mono_address: Address =
                    process.get_module_address("mono-2.0-bdwgc.dll").unwrap();

                if cfg!(debug_assertions) {
                    asr::print_limited::<18>(&format_args!("0x{:x?}", mono_address));
                }

                let mut current_state = State::default();

                loop {
                    //Load each of the relevant values from memory in to variables
                    let timer_state = &asr::timer::state();
                    let old_state = current_state.clone();
                    current_state.update(&process, mono_address);
                    if cfg!(debug_assertions) {
                        current_state.log();
                    }

                    let unstarted_states = [
                        TimerState::NotRunning,
                        TimerState::Ended,
                        TimerState::Unknown,
                    ];

                    if unstarted_states.contains(timer_state)
                        && current_state.should_start(&old_state)
                    {
                        if cfg!(debug_assertions) {
                            asr::print_message("Starting run!");
                        }
                        current_state.zone = Zone::Mountain;
                        if *timer_state == TimerState::Ended {
                            asr::timer::reset();
                        }
                        asr::timer::start();
                    }

                    if *timer_state == TimerState::Running && current_state.should_split() {
                        if cfg!(debug_assertions) {
                            asr::print_limited::<1024>(&format_args!(
                                "Splitting! {:?}",
                                current_state.zone.to_string()
                            ))
                        };
                        asr::timer::split();
                    }

                    next_tick().await;
                }
            })
            .await;
        asr::timer::reset();
    }
}
