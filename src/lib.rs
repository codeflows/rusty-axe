#[derive(Debug)]
pub struct Preset {
    model: &'static str,
    target: Target
}

#[derive(Debug)]
enum Target {
    CurrentEditBuffer,
    BankAndPreset { bank: u8, preset: u8 }
}

pub fn parse_preset(data: &[u8]) -> Option<Preset> {
    let messages = parse_sysex_messages(data);
    return read_syx(messages[0]);
}

const SYSEX_MESSAGE_START_BYTE: u8 = 0xf0;
const SYSEX_MESSAGE_END_BYTE: u8 = 0xf7;

fn parse_sysex_messages(data: &[u8]) -> Vec<&[u8]> {
    let mut messages: Vec<&[u8]> = Vec::new();
    let mut remainder = data;

    while remainder.len() > 0 {
        let start = find_sysex_message_start(remainder).unwrap();
        let end = find_sysex_message_end(remainder).unwrap();
        let boundary = end + 1;
        let message = &remainder[start..boundary];
        messages.push(message);
        remainder = &remainder[boundary..];
    }

    return messages;
}

fn find_sysex_message_start(data: &[u8]) -> Option<usize> {
    data.get(0).and_then(|&byte| {
        if byte == SYSEX_MESSAGE_START_BYTE {
            return Some(0);
        } else {
            return None;
        }
    })
}

fn find_sysex_message_end(data: &[u8]) -> Option<usize> {
    for (index, &byte) in data.iter().enumerate() {
        if byte == SYSEX_MESSAGE_END_BYTE {
            return Some(index);
        }
    }
    return None;
}

fn read_syx(buf: &[u8]) -> Option<Preset> {
    let model = axe_model_name(buf[4]);

    if !validate_header(&buf) {
        println!("This does not look like a Axe FX patch file.");
        print_bytes(buf);
        return None;
    }

    let (file_checksum, calculated_checksum) = get_checksums(&buf);
    if file_checksum != calculated_checksum {
        println!("Invalid checksum (model {})! Expected {:03$X} but got {:03$X}", model, calculated_checksum, file_checksum, 2);
        return None;
    }

    let target: Target;
    if buf[6] == 0x7f {
        target = Target::CurrentEditBuffer;
    } else {
        target = Target::BankAndPreset { bank: buf[6], preset: buf[7] }
    }

    return Some(Preset {
        model: model,
        target: target
    });
}

fn validate_header(buf: &[u8]) -> bool {
    // "Manufacturer sysex ID byte 0. As of firmware 8.02 this is always 00."
    buf[1] == 0x00 &&
    // "Manufacturer sysex ID byte 1. As of firmware 10.02, this is always 01 (in previous firmware versions this was 00).""
    buf[2] == 0x01 &&
    // "Manufacture sysex ID byte 2. As of firmware 10.02, this is 74 (in previous firmware versions this was 7D).""
    buf[3] == 0x74 &&
    (
        // this seems to be the default
        buf[5] == 0x77 ||
        // MIDI_START_IR_DOWNLOAD
        buf[5] == 0x7a ||
        // MIDI_PATCH_DUMP? standard and ultra patches?
        buf[5] == 0x04
    )
}

fn get_checksums(buf: &[u8]) -> (u8, u8) {
    let checksum_index = buf.len() - 2;
    let file_checksum = buf[checksum_index];
    let xor = buf[..checksum_index]
        .iter()
        .fold(0, |acc, &x| acc ^ x);
    let calculated_checksum = xor & 0x7F;
    return (file_checksum, calculated_checksum);
}

fn axe_model_name(code: u8) -> &'static str {
    match code {
        0x00 => "Axe-Fx Standard",
        0x01 => "Axe-Fx Ultra",
        0x03 => "Axe-Fx II",
        0x05 => "FX8",
        0x06 => "Axe-Fx II XL",
        0x07 => "Axe-Fx II XL+",
        0x08 => "AX8",
        _    => "Unknown"
    }
}

fn print_bytes(buf: &[u8]) {
    for b in buf.iter() {
        print!("{:01$X} ", b, 2);
    }
    println!("");
}
