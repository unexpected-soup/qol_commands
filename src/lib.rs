use weft_sdk::*;

declare_mod!(deps: ["base"], init: init);

mod homes;

const SPEED_MOD_NAME: &str = "qol_commands:speed";
const REACH_MOD_NAME: &str = "qol_commands:reach";

const MAX_SPEED_LEVEL: i64 = 10;
const MAX_REACH_LEVEL: i64 = 10;

value! {
    pub struct LastDeath { pub pos: [f64; 3], pub dim_key: u64, pub set: u64 }
}
pub static LAST_DEATH: Component<LastDeath> = Component::named("qol_commands:last_death");

fn init(r: &mut Registry) {
    homes::register(r);
    register_back(r);
    register_misc(r);
}

fn register_back(r: &mut Registry) {
    r.entity_component(&LAST_DEATH, LastDeath { pos: [0.0; 3], dim_key: 0, set: 0 }, component_flags::SAVE | component_flags::KEEP);
    r.attach(&PLAYER, &LAST_DEATH);

    r.on(&ENTITY_DIED, |b| {
        let w = b.world();
        for d in b.iter() {
            // Only players have the component, so mobs are skipped
            if let Some(l) = w.component_mut(d.entity, &LAST_DEATH) {
                *l = LastDeath { pos: d.pos, dim_key: dimension_key(w.dimension()), set: 1 };
            }
        }
    });

    r.command("back", "", |cmd| {
        let (w, p) = (cmd.world(), cmd.player());
        let Some(l) = w.component_mut(p, &LAST_DEATH) else { return };

        if l.set == 0 {
            return cmd.reply("command.qol_commands.back.unset", &[]);
        }

        let Some(dim) = dimension_of_key(l.dim_key) else {
            // THe dimension's mod was removed
            l.set = 0;
            return cmd.reply("command.qol_commands.back.missing_dimension", &[]);
        };

        let target =
            if w.dimension() == dim { None } else { Some(Dimension::generated(name_of(abi::entry_kind::DIMENSION, dim).to_string())) };
        w.teleport(p, target, l.pos);

        cmd.reply("command.qol_commands.back.done", &[]);
        // Keep `l.set == 1` so the player can teleport multiple times to the death spot
    })
    .permission(permission::EVERYONE);
}

fn register_misc(r: &mut Registry) {
    r.command("clear", "[who:targets] | [item:item] | <who:targets> [item:item]", |cmd| {
        let w = cmd.world();

        let mut targets = cmd.targets("who");
        if targets.is_empty() {
            targets.push(cmd.player());
        }

        let item = cmd.id("item");
        let mut total: u32 = 0;

        for id in &targets {
            let inv = w.inventory(*id);
            for i in 0..inv.size() {
                let slot = inv.get(i);
                if item.is_some_and(|filter| filter != slot.item) {
                    continue;
                }
                total += slot.count;
                inv.set(i, ItemStack::EMPTY);
            }
        }

        cmd.reply("command.qol_commands.clear.done", &[Arg::Int(total as i64), Arg::Int(targets.len() as i64)]);
    });

    r.command("speed", "<level:int> [who:targets]", |cmd| {
        let w = cmd.world();
        let level = cmd.int("level").unwrap_or(1).clamp(1, MAX_SPEED_LEVEL);

        let mut targets = cmd.targets("who");
        if targets.is_empty() {
            targets.push(cmd.player());
        }

        for id in &targets {
            w.add_modifier(*id, &attribute::MOVEMENT_SPEED, SPEED_MOD_NAME, attr_op::ADD, level as f64 - 1.0, 0);
        }

        cmd.reply("command.qol_commands.speed.done", &[Arg::Int(level), Arg::Int(targets.len() as i64)]);
    });

    r.command("reach", "<level:int> [who:targets]", |cmd| {
        let w = cmd.world();
        let level = cmd.int("level").unwrap_or(1).clamp(1, MAX_REACH_LEVEL);

        let mut targets = cmd.targets("who");
        if targets.is_empty() {
            targets.push(cmd.player());
        }

        for id in &targets {
            w.add_modifier(*id, &attribute::BLOCK_REACH, REACH_MOD_NAME, attr_op::ADD, level as f64 - 1.0, 0);
        }

        cmd.reply("command.qol_commands.reach.done", &[Arg::Int(level), Arg::Int(targets.len() as i64)]);
    });
}
