module ControlDatapath {
    input clk: bit;
    input rst: bit;
    input start: bit;
    input opcode: u4;
    input mode: u2;
    input data: u8;
    input key: u8;

    output result: u8;
    output high: u4;
    output low: u4;
    output busy: bit;
    output done: bit;
    output zero: bit;
    output carry_like: bit;
    output status: u4;

    reg acc: u8;
    reg state: u3;
    reg phase: u8;
    reg flags: u4;

    wire opcode_low: u2;
    wire mode_mix: u8;
    wire data_mix: u8;
    wire selected: u8;
    wire folded: u8;
    wire next_acc: u8;
    wire next_flags: u4;

    const idle: u3 = 0;
    const load: u3 = 1;
    const run: u3 = 2;
    const finish: u3 = 3;
    const mask_low: u8 = 0x0f;
    const mask_high: u8 = 0xf0;

    opcode_low = opcode[1:0];
    mode_mix = case mode {
        0 => data,
        1 => data ^ key,
        2 => (data & mask_low) + (key & mask_high),
        else => data + key
    };
    data_mix = if opcode[3] {
        mode_mix ^ (phase + key)
    } else {
        mode_mix + phase
    };
    selected = case opcode_low {
        0 => data_mix,
        1 => data_mix ^ acc,
        2 => data_mix + acc,
        else => (data_mix & key) | (acc & ~key)
    };
    folded = selected ^ (selected >> 4);
    next_acc = case state {
        0 => if start { data_mix } else { acc },
        1 => selected,
        2 => if phase[0] { selected + folded } else { selected ^ folded },
        else => acc
    };
    next_flags = case opcode_low {
        0 => selected[3:0],
        1 => selected[7:4],
        2 => folded[3:0],
        else => phase[3:0]
    };

    result = acc;
    high = acc[7:4];
    low = acc[3:0];
    busy = state != idle;
    done = state == finish;
    zero = acc == 0;
    carry_like = (selected < acc) | folded[7];
    status = flags ^ phase[3:0];

    on rising(clk) {
        acc = if rst { 0 } else { next_acc };
        flags = if rst { 0 } else { next_flags };
        phase = if rst {
            0
        } else {
            if state == run {
                phase + 1
            } else {
                phase
            }
        };
        state = if rst {
            idle
        } else {
            case state {
                0 => if start { load } else { idle },
                1 => run,
                2 => if phase == 7 { finish } else { run },
                3 => idle,
                else => idle
            }
        };
    }
}
