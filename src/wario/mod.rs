use {
    smash::{
        lua2cpp::L2CFighterCommon,
        phx::Hash40,
        app::{lua_bind::*, *},
        lib::{lua_const::*, L2CValue}
    },
    smash_script::*,
    smashline::*,
    table_const::*,
};

// I overwrite both FIGHTER_WARIO_STATUS_KIND_SPECIAL_S_START and FIGHTER_WARIO_STATUS_KIND_SPECIAL_S_SEARCH
// This is because for whatever reason, Wario *doesn't* use FIGHTER_STATUS_KIND_SPECIAL_S.

#[status_script(agent = "wario", status = FIGHTER_WARIO_STATUS_KIND_SPECIAL_S_START, condition = LUA_SCRIPT_STATUS_FUNC_STATUS_PRE)]
unsafe fn wario_special_s_start_pre(fighter: &mut L2CFighterCommon) -> L2CValue {
    wario_special_s_pre_inner(fighter)
}

#[status_script(agent = "wario", status = FIGHTER_WARIO_STATUS_KIND_SPECIAL_S_SEARCH, condition = LUA_SCRIPT_STATUS_FUNC_STATUS_PRE)]
unsafe fn wario_special_s_search_pre(fighter: &mut L2CFighterCommon) -> L2CValue {
    wario_special_s_pre_inner(fighter)
}

unsafe fn wario_special_s_pre_inner(fighter: &mut L2CFighterCommon) -> L2CValue {
    StatusModule::init_settings(
        fighter.module_accessor,
        SituationKind(*SITUATION_KIND_NONE),
        *FIGHTER_KINETIC_TYPE_UNIQ,
        *GROUND_CORRECT_KIND_KEEP as u32,
        GroundCliffCheckKind(*GROUND_CLIFF_CHECK_KIND_NONE),
        true,
        *FIGHTER_STATUS_WORK_KEEP_FLAG_NONE_FLAG,
        *FIGHTER_STATUS_WORK_KEEP_FLAG_NONE_INT,
        *FIGHTER_STATUS_WORK_KEEP_FLAG_NONE_FLOAT,
        0
    );
    FighterStatusModuleImpl::set_fighter_status_data(
        fighter.module_accessor,
        false,
        *FIGHTER_TREADED_KIND_NO_REAC,
        false,
        false,
        false,
        (*FIGHTER_LOG_MASK_FLAG_ATTACK_KIND_SPECIAL_S | *FIGHTER_LOG_MASK_FLAG_ACTION_CATEGORY_ATTACK | *FIGHTER_LOG_MASK_FLAG_ACTION_TRIGGER_ON) as u64,
        *FIGHTER_STATUS_ATTR_START_TURN as u32,
        *FIGHTER_POWER_UP_ATTACK_BIT_SPECIAL_S as u32,
        0
    );
    0.into()
}

#[status_script(agent = "wario", status = FIGHTER_WARIO_STATUS_KIND_SPECIAL_S_START, condition = LUA_SCRIPT_STATUS_FUNC_STATUS_MAIN)]
unsafe fn wario_special_s_start_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    wario_special_s_main_inner(fighter)
}

#[status_script(agent = "wario", status = FIGHTER_WARIO_STATUS_KIND_SPECIAL_S_SEARCH, condition = LUA_SCRIPT_STATUS_FUNC_STATUS_MAIN)]
unsafe fn wario_special_s_search_main(fighter: &mut L2CFighterCommon) -> L2CValue {
    wario_special_s_main_inner(fighter)
}

unsafe fn wario_special_s_main_inner(fighter: &mut L2CFighterCommon) -> L2CValue {
    // This section only runs once
    MotionModule::change_motion(
        fighter.module_accessor,
        Hash40::new("attack_dash"),
        0.0,
        1.0,
        false,
        0.0,
        false,
        false
    );
    if fighter.global_table[SITUATION_KIND].get_i32() != *SITUATION_KIND_GROUND {
        // Force downwards motion for the air version.
        GroundModule::correct(fighter.module_accessor, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
        KineticModule::change_kinetic(fighter.module_accessor, *FIGHTER_KINETIC_TYPE_MOTION_AIR_ANGLE);
        let lr = PostureModule::lr(fighter.module_accessor);
        // Set's Wario's movement at a downwards angle of 20 degrees, multiplied by your facing direction.
        // Which is then converted to radians. Ugh, radians.
        sv_kinetic_energy!(
            set_angle,
            fighter,
            FIGHTER_KINETIC_ENERGY_ID_MOTION,
            (-20.0 * lr).to_radians()
        );
        // Multiplies how far the animation movement moves you.
        sv_kinetic_energy!(
            set_speed_mul,
            fighter,
            FIGHTER_KINETIC_ENERGY_ID_MOTION,
            0.7
        );
    }
    else {
        // Lets you run off of ledges with Side Special. If you *don't* want this,
        // change GROUND_CORRECT_KIND_GROUND to GROUND_CORRECT_KIND_GROUND_CLIFF_STOP
        GroundModule::correct(fighter.module_accessor, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND));
        KineticModule::change_kinetic(fighter.module_accessor, *FIGHTER_KINETIC_TYPE_MOTION);
    }
    // Exclusively handles Wario's color flashing
    if !StopModule::is_stop(fighter.module_accessor) {
        wario_special_s_substatus(fighter, false.into());
    }
    fighter.global_table[SUB_STATUS].assign(&L2CValue::Ptr(wario_special_s_substatus as *const () as _));
    fighter.sub_shift_status_main(L2CValue::Ptr(wario_special_s_main_loop as *const () as _))
}

unsafe extern "C" fn wario_special_s_substatus(fighter: &mut L2CFighterCommon, _param_1: L2CValue) -> L2CValue {
    if [5.0, 20.0].contains(&MotionModule::frame(fighter.module_accessor)){
        macros::COL_NORMAL(fighter);
        macros::FLASH(fighter, 1.0, 0.0, 0.0, 0.5);
    } else if [10.0, 25.0].contains(&MotionModule::frame(fighter.module_accessor)){
        macros::COL_NORMAL(fighter);
        macros::FLASH(fighter, 0.0, 0.0, 1.0, 0.5);
    } else if [15.0].contains(&MotionModule::frame(fighter.module_accessor)){
        macros::COL_NORMAL(fighter);
        macros::FLASH(fighter, 1.0, 1.0, 0.0, 0.5);
    } else if MotionModule::frame(fighter.module_accessor) > 25.0{
        macros::COL_NORMAL(fighter);
    };
    0.into()
}

unsafe extern "C" fn wario_special_s_main_loop(fighter: &mut L2CFighterCommon) -> L2CValue {
    // This function runs every frame.

    // Store if you've landed the attack as well as your current situation in local variables
    let infliction = AttackModule::is_infliction_status(fighter.module_accessor, *COLLISION_KIND_MASK_ALL);
    let situation = fighter.global_table[SITUATION_KIND].get_i32();

    // This effectively goes unused, but my initial idea was to let the dash attack animation play out
    // until the end. That didn't stay, so this effectively goes unused.
    if CancelModule::is_enable_cancel(fighter.module_accessor)
    && infliction {
        if fighter.sub_wait_ground_check_common(false.into()).get_bool()
        || fighter.sub_air_check_fall_common().get_bool() {
            return 1.into();
        }
    }

    // Checks if your situation kind has changed on the frame this function is run.
    if StatusModule::is_situation_changed(fighter.module_accessor) {
        // If the attack is over...
        if MotionModule::frame(fighter.module_accessor) > 25.0 {
            let status = if situation == *SITUATION_KIND_GROUND {
                // Go into special fall landing if landing on the ground
                FIGHTER_STATUS_KIND_LANDING_FALL_SPECIAL
            }
            else {
                // Go into regular fall if you get pushed off the ground somehow
                FIGHTER_STATUS_KIND_FALL
            };
            fighter.change_status(status.into(), false.into());
            return 0.into();
        }
        else {
            // if the attack *isn't* over...
            if situation == *SITUATION_KIND_GROUND {
                // Change the kinetic to regular MOTION instead of MOTION_AIR_ANGLE, then place him on the ground.
                KineticModule::change_kinetic(fighter.module_accessor, *FIGHTER_KINETIC_TYPE_MOTION);
                GroundModule::correct(fighter.module_accessor, GroundCorrectKind(*GROUND_CORRECT_KIND_GROUND));
            }
            else {
                // Simply have him placed in the air. Like with above, change this if you don't
                // want him running off of ledges.
                GroundModule::correct(fighter.module_accessor, GroundCorrectKind(*GROUND_CORRECT_KIND_AIR));
            }
        }
    }

    // If Wario is attacking, has hit someone, and is currently not in hitstop of any kind...
    if MotionModule::frame(fighter.module_accessor) < 25.0
    && infliction
    && !fighter.global_table[IS_STOP].get_bool() {
        if situation == *SITUATION_KIND_AIR {
            // If you're in the air, have him skip to frame 25 and run the animation at 0.1x speed...
            MotionModule::change_motion(fighter.module_accessor, Hash40::new("attack_dash"), 25.0, 0.1, false, 0.0, false, false);
            // ... change his kinetic type to FALL, so you can control his drift...
            KineticModule::change_kinetic(fighter.module_accessor, *FIGHTER_KINETIC_TYPE_FALL);
            let stop_rise  = smash::phx::Vector3f { x: 0.0, y: 1.0, z: 1.0 };
			KineticModule::mul_speed(fighter.module_accessor, &stop_rise, *FIGHTER_KINETIC_ENERGY_ID_CONTROL);
            // ... then set his speed so he bounces off of what he hit.
            let lr = PostureModule::lr(fighter.module_accessor);
            sv_kinetic_energy!(
                set_speed,
                fighter,
                FIGHTER_KINETIC_ENERGY_ID_CONTROL,
                -0.8 * lr
            );
            sv_kinetic_energy!(
                set_speed,
                fighter,
                FIGHTER_KINETIC_ENERGY_ID_GRAVITY,
                2.0
            );
        }
        else {
            // On the ground just make him skip to frame 25 and run at 0.5x speed.
            MotionModule::change_motion(fighter.module_accessor, Hash40::new("attack_dash"), 25.0, 0.5, false, 0.0, false, false);
        }
    }

    // If your frame is over frame 28 and you're in the air...
    if situation == *SITUATION_KIND_AIR && MotionModule::frame(fighter.module_accessor) > 28.0 {
        let status = if infliction {
            // If you hit something, become actionable as you fall.
            FIGHTER_STATUS_KIND_FALL
        }
        else {
            // If you whiffed, die.
            FIGHTER_STATUS_KIND_FALL_SPECIAL
        };
        // Change status depending on hit or whiff.
        fighter.change_status(status.into(), false.into());
    }
    // Similarly, if your frame is over frame 40 and you're on the ground...
    if situation == *SITUATION_KIND_GROUND && MotionModule::frame(fighter.module_accessor) > 40.0 {
        // Just go into wait lmao
        fighter.change_status(FIGHTER_STATUS_KIND_WAIT.into(), false.into());
    }
    // End script
    0.into()
}

pub fn install() {
    install_status_scripts!(
        wario_special_s_start_pre, wario_special_s_search_pre,
        wario_special_s_start_main, wario_special_s_search_main
    );
}