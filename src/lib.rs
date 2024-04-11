#![no_std]

use asr::deep_pointer::DeepPointer;
use asr::timer::TimerState;
use asr::{future::next_tick, Address, Process};

asr::async_main!(stable);
asr::panic_handler!();

#[derive(Default, Clone)]
struct State {
    left_hand_grabbed_surface: u32,
    right_hand_grabbed_surface: u32,
    position_x: f32,
    position_y: f32,

    zone: u8,
}

impl State {
    fn generate(process: &Process, zone: u8) -> Result<Option<State>, asr::Error> {
        const LEFT_HAND_GRABBED_SURFACE_PATH: &[u64] = &[0x20, 0xA0, 0x34];
        const RIGHT_HAND_GRABBED_SURFACE_PATH: &[u64] = &[0x18, 0xA0, 0x34];
        const POSITION_X_PATH: &[u64] = &[0xE0];
        const POSITION_Y_PATH: &[u64] = &[0xE4];

        #[cfg(debug_assertions)]
        asr::print_message("Getting anim controller");
        let Some(animation_controller) = identify_valid_anim_controller(process)? else {
            return Ok(None);
        };

        #[cfg(debug_assertions)]
        asr::print_message("Getting position object");
        let Some(position_object) = identify_valid_position_object(process)? else {
            return Ok(None);
        };

        #[cfg(debug_assertions)]
        asr::print_message("Getting state vars");

        #[cfg(debug_assertions)]
        asr::print_message("Getting pos x");
        let position_x = process.read_pointer_path::<f32>(
            position_object,
            asr::PointerSize::Bit64,
            POSITION_X_PATH,
        )?;

        #[cfg(debug_assertions)]
        asr::print_message("Getting pos y");
        let position_y = process.read_pointer_path::<f32>(
            position_object,
            asr::PointerSize::Bit64,
            POSITION_Y_PATH,
        )?;

        #[cfg(debug_assertions)]
        asr::print_message("Getting left hand grab surface");
        let left_hand_grabbed_surface = process
            .read_pointer_path::<u32>(
                animation_controller,
                asr::PointerSize::Bit64,
                LEFT_HAND_GRABBED_SURFACE_PATH,
            )
            .unwrap_or(0);

        #[cfg(debug_assertions)]
        asr::print_message("Getting right hand grab surface");
        let right_hand_grabbed_surface = process
            .read_pointer_path::<u32>(
                animation_controller,
                asr::PointerSize::Bit64,
                RIGHT_HAND_GRABBED_SURFACE_PATH,
            )
            .unwrap_or(0);

        Ok(Some(State {
            left_hand_grabbed_surface,
            right_hand_grabbed_surface,
            position_x,
            position_y,
            zone,
        }))
    }

    fn log(&self) {
        asr::print_limited::<1024>(&format_args!(
            "({:.2?}, {:.2?}) - ({:x?}, {:x?}) - Zone:  {:?}",
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
        const MOUNTAIN: u8 = 1;
        const JUNGLE: u8 = 1 << 1;
        const FACTORY: u8 = 1 << 2;
        const POOL: u8 = 1 << 3;
        const CONSTRUCTION: u8 = 1 << 4;
        const CAVE: u8 = 1 << 5;
        const ICE: u8 = 1 << 6;
        const CREDITS: u8 = 1 << 7;

        if (self.zone & MOUNTAIN == 0) && self.position_y > 31f32 {
            self.zone |= MOUNTAIN;
            return true;
        }
        if (self.zone & JUNGLE == 0) && self.position_y > 55f32 && self.position_x < 0f32 {
            self.zone |= JUNGLE;
            return true;
        }
        if (self.zone & FACTORY == 0)
            && self.position_y > 80f32
            && self.position_y < 87f32
            && self.position_x > 8f32
        {
            self.zone |= FACTORY;
            return true;
        }
        if (self.zone & POOL == 0) && self.position_y > 109f32 && self.position_x < 20f32 {
            self.zone |= POOL;
            return true;
        }
        if (self.zone & CONSTRUCTION == 0) && self.position_y > 135f32 {
            self.zone |= CONSTRUCTION;
            return true;
        }
        if (self.zone & CAVE == 0) && self.position_y > 152f32 {
            self.zone |= CAVE;
            return true;
        }
        if (self.zone & ICE == 0) && self.position_y > 204f32 && self.position_x < 47f32 {
            self.zone |= ICE;
            return true;
        }
        if (self.zone & CREDITS == 0) && self.position_y > 247f32 {
            self.zone |= CREDITS;
            return true;
        }

        false
    }

    fn should_reset(&self) -> bool {
        self.position_y < -3f32
    }
}

fn identify_valid_anim_controller(process: &Process) -> Result<Option<Address>, asr::Error> {
    let base_address = process.get_module_address("UnityPlayer.dll")?;
    let anim_controller_pointer_list: &[DeepPointer<12>] = &[
        DeepPointer::new(
            base_address,
            asr::PointerSize::Bit64,
            &[0x1AD8388, 0x0, 0x3A0, 0x0, 0x10, 0x30, 0x38, 0x28],
        ),
        DeepPointer::new(
            base_address,
            asr::PointerSize::Bit64,
            &[0x1A8C3C0, 0x328, 0x50, 0x168, 0x30, 0x78, 0x30, 0x38, 0x28],
        ),
        DeepPointer::new(
            base_address,
            asr::PointerSize::Bit64,
            &[
                0x1AD81D0, 0x940, 0x1F8, 0x298, 0x150, 0x68, 0x68, 0x38, 0x28,
            ],
        ),
        DeepPointer::new(
            base_address,
            asr::PointerSize::Bit64,
            &[
                0x1AD8388, 0x0, 0x3A0, 0x0, 0x28, 0x10, 0x0, 0x30, 0x38, 0x28,
            ],
        ),
        DeepPointer::new(
            base_address,
            asr::PointerSize::Bit64,
            &[
                0x1A8C3C0, 0xF0, 0xE8, 0x50, 0x168, 0x30, 0x78, 0x30, 0x38, 0x28,
            ],
        ),
        DeepPointer::new(
            base_address,
            asr::PointerSize::Bit64,
            &[
                0x1A8C280, 0x50, 0x128, 0xE8, 0x78, 0xC8, 0x30, 0x30, 0x38, 0x28,
            ],
        ),
        DeepPointer::new(
            base_address,
            asr::PointerSize::Bit64,
            &[
                0x1A8C3C0, 0x2A0, 0x0, 0x18, 0x10, 0xD0, 0x30, 0x30, 0x38, 0x28,
            ],
        ),
        DeepPointer::new(
            base_address,
            asr::PointerSize::Bit64,
            &[0x1B15160, 0x8, 0x8, 0x28, 0x0, 0x10, 0x30, 0x30, 0x38, 0x28],
        ),
        DeepPointer::new(
            base_address,
            asr::PointerSize::Bit64,
            &[
                0x1AD81C8, 0x40, 0xD78, 0xC8, 0x298, 0x150, 0x68, 0x68, 0x38, 0x28,
            ],
        ),
        DeepPointer::new(
            base_address,
            asr::PointerSize::Bit64,
            &[
                0x1AD8388, 0x0, 0x5F8, 0x3B0, 0x0, 0x10, 0x30, 0x158, 0x48, 0x28,
            ],
        ),
    ];

    #[cfg(debug_assertions)]
    asr::print_message("Checking Animation Controllers");
    for anim_controller_pointer in anim_controller_pointer_list.iter() {
        let Ok(anim_controller) = anim_controller_pointer.deref::<u64>(process) else {
            continue;
        };

        #[cfg(debug_assertions)]
        asr::print_message("Checking left_strength");
        let Ok(left_strength) = process.read_pointer_path::<f32>(
            anim_controller,
            asr::PointerSize::Bit64,
            &[0x20, 0x18, 0x18, 0x18, 0xC0],
        ) else {
            continue;
        };

        #[cfg(debug_assertions)]
        asr::print_message("Return good anim_controller if matches");
        if left_strength == 75f32 {
            return Ok(Some(Address::new(anim_controller)));
        }
    }

    #[cfg(debug_assertions)]
    asr::print_message("Failed To Find Animation Controller");
    Ok(None)
}

fn identify_valid_position_object(process: &Process) -> Result<Option<Address>, asr::Error> {
    let base_address = process.get_module_address("UnityPlayer.dll")?;
    let position_object_pointer_list: &[(DeepPointer<9>, u64)] = &[
        (
            DeepPointer::new(
                base_address,
                asr::PointerSize::Bit64,
                &[0x1A8C3C0, 0x328, 0x78, 0xC8, 0x30, 0x30, 0x48],
            ),
            0x0,
        ),
        (
            DeepPointer::new(
                base_address,
                asr::PointerSize::Bit64,
                &[0x1AD8388, 0x0, 0x3A0, 0x0, 0x10, 0x30, 0x48, 0x90],
            ),
            0x0,
        ),
        (
            DeepPointer::new(
                base_address,
                asr::PointerSize::Bit64,
                &[0x1A8C3C0, 0x328, 0x78, 0xC8, 0x140, 0x30, 0x30, 0x48],
            ),
            0x0,
        ),
        (
            DeepPointer::new(
                base_address,
                asr::PointerSize::Bit64,
                &[0x1AD8388, 0x0, 0x3A0, 0x0, 0x10, 0x30, 0x168, 0x80],
            ),
            0x0,
        ),
        (
            DeepPointer::new(
                base_address,
                asr::PointerSize::Bit64,
                &[0x1A8C3C0, 0x328, 0x78, 0xC8, 0x30, 0x30, 0x48, 0x90],
            ),
            0x0,
        ),
        (
            DeepPointer::new(
                base_address,
                asr::PointerSize::Bit64,
                &[0x1A8C3C0, 0x328, 0x78, 0xC8, 0x30, 0x30, 0x48, 0x90],
            ),
            0x0,
        ),
        (
            DeepPointer::new(
                base_address,
                asr::PointerSize::Bit64,
                &[0x1B15160, 0x8, 0x8, 0x28, 0x0, 0xA0, 0x18, 0x0],
            ),
            0x10,
        ),
        (
            DeepPointer::new(
                base_address,
                asr::PointerSize::Bit64,
                &[0x1B15168, 0x8, 0x48, 0x28, 0x0, 0xA0, 0x18, 0x0],
            ),
            0x10,
        ),
        (
            DeepPointer::new(
                base_address,
                asr::PointerSize::Bit64,
                &[0x1B1AAF8, 0x0, 0x88, 0x28, 0x0, 0xA0, 0x18, 0x0],
            ),
            0x10,
        ),
        (
            DeepPointer::new(
                base_address,
                asr::PointerSize::Bit64,
                &[0x1A8C3C0, 0xF0, 0xE8, 0x78, 0xC8, 0x30, 0x30, 0x48, 0xE0],
            ),
            0x0,
        ),
    ];

    #[cfg(debug_assertions)]
    asr::print_message("Checking Position Objects");
    for position_object_pointer in position_object_pointer_list.iter() {
        let Ok(position_object) = position_object_pointer.0.deref(process) else {
            continue;
        };
        let Ok(pos_z) = process.read_pointer_path::<f32>(
            position_object,
            asr::PointerSize::Bit64,
            &[position_object_pointer.1 + 0xE8],
        ) else {
            continue;
        };

        if pos_z == -0.5f32 {
            return Ok(Some(Address::new(position_object)));
        }
    }

    #[cfg(debug_assertions)]
    asr::print_message("Failed To Find Position Object");
    Ok(None)
}

async fn main() {
    loop {
        //Attempt to attach to process with the following name
        let process = Process::wait_attach("A Difficult Game About Climbing.exe").await;
        process
            .until_closes(async {
                asr::print_message("Creating Initial State");

                let mut current_state = State::default();

                loop {
                    //Load each of the relevant values from memory in to variables
                    let timer_state = &asr::timer::state();
                    let old_state = current_state.clone();
                    current_state = match State::generate(&process, current_state.zone) {
                        Ok(Some(state)) => state,
                        Ok(None) => {
                            continue;
                        }
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

                    //Logic to trigger timer start if necessary
                    if unstarted_states.contains(timer_state)
                        && current_state.should_start(&old_state)
                    {
                        #[cfg(debug_assertions)]
                        asr::print_message("Starting run!");
                        asr::timer::start();
                        current_state.zone = 0;
                    }

                    //Logic to trigger splits
                    if *timer_state == TimerState::Running && current_state.should_split() {
                        #[cfg(debug_assertions)]
                        asr::print_limited::<1024>(&format_args!(
                            "Splitting! {:?}",
                            current_state.zone
                        ));

                        asr::timer::split();
                    }

                    //Logic to trigger Resets
                    if *timer_state != TimerState::NotRunning && current_state.should_reset() {
                        #[cfg(debug_assertions)]
                        asr::print_message("Reseting Run");

                        current_state.zone = 0;
                        asr::timer::reset();
                    }

                    //Logic to reset active zones to 0 so fresh splits can work
                    if *timer_state == TimerState::NotRunning {
                        current_state.zone = 0;
                    }

                    next_tick().await;
                }
            })
            .await;
    }
}
