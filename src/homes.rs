use weft_sdk::*;

const MAX_HOMES: usize = 8;
const NAME_LEN: usize = 16;
const DEFAULT_NAME: &str = "home";

value! {
    struct Home { pos: [f64; 3], dim_key: u64, name: [u8; NAME_LEN], set: u64 }
}
value! {
    struct Homes { slots: [Home; MAX_HOMES] }
}
static HOMES: Component<Homes> = Component::named("qol_commands:homes");

pub fn register(r: &mut Registry) {
    let empty = Home { pos: [0.0; 3], dim_key: 0, name: [0; NAME_LEN], set: 0 };
    r.entity_component(&HOMES, Homes { slots: [empty; MAX_HOMES] }, component_flags::SAVE | component_flags::KEEP);
    r.attach(&PLAYER, &HOMES);

    r.command("sethome", "[name:word]", cmd_sethome)
        .permission(permission::EVERYONE);
    r.command("home", "[name:word]", cmd_home)
        .permission(permission::EVERYONE);
    r.command("delhome", "<name:word>", cmd_delhome)
        .permission(permission::EVERYONE);
    r.command("homes", "", cmd_homes)
        .permission(permission::EVERYONE);
}

fn cmd_sethome(cmd: &mut ServerCommand) {
    let w = cmd.world();
    let p = cmd.player();
    let name = cmd.text("name").unwrap_or(DEFAULT_NAME).to_string();
    let key = pack_home_name(&name);
    let Some(st) = w.entity(p) else { return };
    let Some(homes) = w.component_mut(p, &HOMES) else { return };

    // Overwrite a home with the same name, otherwise take the first free slot.
    let slot = homes
        .slots
        .iter()
        .position(|s| s.set != 0 && s.name == key)
        .or_else(|| homes.slots.iter().position(|s| s.set == 0));
    let Some(i) = slot else {
        return cmd.reply("command.qol_commands.sethome.full", &[Arg::Int(MAX_HOMES as i64)]);
    };
    homes.slots[i] = Home { pos: st.pos, dim_key: dimension_key(w.dimension()), name: key, set: 1 };
    cmd.reply("command.qol_commands.sethome.done", &[Arg::Text(&name)]);
}

fn cmd_home(cmd: &mut ServerCommand) {
    let w = cmd.world();
    let p = cmd.player();
    let name = cmd.text("name").unwrap_or(DEFAULT_NAME).to_string();
    let key = pack_home_name(&name);
    let Some(homes) = w.component_mut(p, &HOMES) else { return };
    let Some(h) = homes.slots.iter_mut().find(|s| s.set != 0 && s.name == key) else {
        return cmd.reply("command.qol_commands.home.unset", &[Arg::Text(&name)]);
    };
    let Some(dim) = dimension_of_key(h.dim_key) else {
        // The dimension's mod was removed
        h.set = 0;
        return cmd.reply("command.qol_commands.home.missing_dimension", &[Arg::Text(&name)]);
    };
    let target = if w.dimension() == dim { None } else { Some(Dimension::generated(name_of(abi::entry_kind::DIMENSION, dim).to_string())) };
    w.teleport(p, target, h.pos);
    cmd.reply("command.qol_commands.home.done", &[Arg::Text(&name)]);
}

fn cmd_delhome(cmd: &mut ServerCommand) {
    let w = cmd.world();
    let p = cmd.player();
    let name = cmd.text("name").unwrap_or(DEFAULT_NAME).to_string();
    let key = pack_home_name(&name);
    let Some(homes) = w.component_mut(p, &HOMES) else { return };
    match homes.slots.iter_mut().find(|s| s.set != 0 && s.name == key) {
        Some(h) => {
            h.set = 0;
            cmd.reply("command.qol_commands.delhome.done", &[Arg::Text(&name)]);
        }
        None => cmd.reply("command.qol_commands.home.unset", &[Arg::Text(&name)]),
    }
}

fn cmd_homes(cmd: &mut ServerCommand) {
    let w = cmd.world();
    let Some(homes) = w.component(cmd.player(), &HOMES) else { return };
    let names: Vec<&str> = homes
        .slots
        .iter()
        .filter(|s| s.set != 0)
        .map(|s| unpack_home_name(&s.name))
        .collect();
    if names.is_empty() {
        cmd.reply("command.qol_commands.homes.none", &[]);
    } else {
        cmd.reply("command.qol_commands.homes.list", &[Arg::Text(&names.join(", "))]);
    }
}

/// Transform to lowecase and cut to NAME_LEN
fn pack_home_name(name: &str) -> [u8; NAME_LEN] {
    let mut out = [0u8; NAME_LEN];
    let mut len = 0;
    for c in name.to_lowercase().chars() {
        let n = c.len_utf8();
        if len + n > NAME_LEN {
            break;
        }
        c.encode_utf8(&mut out[len..]);
        len += n;
    }
    out
}

fn unpack_home_name(bytes: &[u8; NAME_LEN]) -> &str {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(NAME_LEN);
    std::str::from_utf8(&bytes[..end]).unwrap_or("")
}
