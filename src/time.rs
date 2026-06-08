use bevy::prelude::*;

#[derive(Resource)]
pub struct WorldTime {
    pub ticks: u64,
    pub day_length: u64,
}

impl WorldTime {
    pub fn new() -> Self {
        Self {
            ticks: 0,
            day_length: 24000,
        }
    }
    
    pub fn update(&mut self) {
        self.ticks += 1;
        if self.ticks >= self.day_length * 8 {
            self.ticks = 0;
        }
    }
    
    pub fn is_night(&self) -> bool {
        let time_of_day = self.ticks % self.day_length;
        time_of_day > 13000 && time_of_day < 23000
    }
    
    pub fn moon_phase(&self) -> MoonPhase {
        let day = self.ticks / self.day_length;
        match day % 8 {
            0 => MoonPhase::New,
            1 => MoonPhase::WaxingCrescent,
            2 => MoonPhase::FirstQuarter,
            3 => MoonPhase::WaxingGibbous,
            4 => MoonPhase::Full,
            5 => MoonPhase::WaningGibbous,
            6 => MoonPhase::LastQuarter,
            7 => MoonPhase::WaningCrescent,
            _ => MoonPhase::New,
        }
    }
    
    pub fn is_full_moon(&self) -> bool {
        matches!(self.moon_phase(), MoonPhase::Full)
    }
}

pub enum MoonPhase {
    New,
    WaxingCrescent,
    FirstQuarter,
    WaxingGibbous,
    Full,
    WaningGibbous,
    LastQuarter,
    WaningCrescent,
}

pub fn update_day_night(mut world_time: ResMut<WorldTime>) {
    world_time.update();
}
