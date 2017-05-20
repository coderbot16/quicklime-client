// Taken from http://minecraft.gamepedia.com/Breaking

enum BaseToolType {
	None,
	Axe,
	Pickaxe,
	Shears,
	Shovel,
	Sword
}

// 0 = None, 1 = Efficiency I, and so on
fn efficiency_bonus(lvl: u32) -> f32 {
	if lvl == 0 {
		0.0
	} else {
		(lvl*lvl as f32) + 1.0
	}
}

fn haste_factor(lvl: u32) -> f32 {
	1.0 + (lvl as f32)*0.2
}

fn fatigue_factor(lvl: u32) -> f32 {
	(0.3).powi(lvl as i32)
}

fn flying_factor() -> f32 {
	0.2
}

fn underwater_factor() -> f32 {
	0.2
}

fn target_accumulator(hardness: f32, tool_good: bool) -> f32 {
	hardness * if tool_good {30.0} else {100.0}
}

// TODO: Delay calculation (5 ticks if it takes more than 1 tick to break a block) - doesn't exist in PE