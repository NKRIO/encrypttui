use encrypttui::layer::Position;
use encrypttui::cfg::{
    LAYERS,
    INPUT_POS,
    INPUT_LEFT,
    INPUT_RIGHT,
    DEBUG,
    CRYPTDEVICE_UUID,
    CRYPTNAME,
    TRY_TIMES,
    TRY_INTERVAL
};

use crossterm::{
    event::{read, Event, KeyCode},
    terminal::{enable_raw_mode, disable_raw_mode},
};

use std::process::{Command, Stdio};
use std::io::{stdout, Write};
use std::path::Path;
use std::{fs, thread, time};


fn read_password(max_show_length: usize) -> String {
    let mut password = String::new();
    let mut stdout = stdout();

    stdout.flush().unwrap();
    enable_raw_mode().unwrap();

    loop {
        if let Event::Key(key_event) = read().unwrap() {
            match key_event.code {
                KeyCode::Enter => {
                    if password.len()>0 {
                        break;
                    }
                },
                KeyCode::Char(c) => {
                    password.push(c);
                    if password.len()<=max_show_length {
                        print!("*");
                        stdout.flush().unwrap();
                    }
                },
                KeyCode::Backspace => {
                    if !password.is_empty() {
                        password.pop();
                        if password.len()<max_show_length {
                            print!("\x08 \x08"); // backspace
                            stdout.flush().unwrap();
                        }
                    }
                },
                _ => {}
            }
        }
    }

    disable_raw_mode().unwrap();

    password
}

fn truncate_visible(s: &str, max_len: u32, start_index: u32) -> String {
    let mut result = String::new();
    let mut visible_count: u32 = 0;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        // encrypttui supports ANSI escaping, `\x1B[m` ONLY
        if c == '\x1B' {
            // ANSI starts
            result.push(c);
            while let Some(&next) = chars.peek() {
                result.push(next);
                chars.next();
                if next == 'm'/* || next == 'K'*/ {
                    break;
                }
            }
        } else {
            if visible_count < max_len {
                if visible_count >= start_index {
                    if c == '\0' {
                        result.push_str("\x1B[1C");
                    } else {
                        result.push(c);
                    }
                }
                visible_count += 1;
            } else {
                break;
            }
        }
    }

    result
}

fn calc_pos(x_or_y: &Position, col_or_row: u16) -> i16 {
    if x_or_y.denominator == 0 {
        return (if x_or_y.flip {col_or_row as i16 - x_or_y.absolute} else {x_or_y.absolute}) as i16;
    }
    let out: i16 = (col_or_row as f64 * x_or_y.numerator as f64 / x_or_y.denominator as f64).ceil() as i16;
    if x_or_y.flip {col_or_row as i16 - 1 - out} else {out}
}
macro_rules! move_to {
    ($col:expr, $row:expr) => {
        print!("\x1B[{};{}H", $row, $col);
    };
}
macro_rules! clear_console {
    () => {
        print!("\x1Bc");
    };
}
macro_rules! clear_attr {
    () => {
        print!("\x1B[0m");
    };
}

fn show_input_screen() -> String {
    let out_password: String;
    clear_console!();

    match crossterm::terminal::size() {
    Ok((col,row)) => {
        for layer in LAYERS.iter() {
            let posx = calc_pos(&layer.position.0, col) - layer.origin.0 as i16;
            let posy = calc_pos(&layer.position.1, row) - layer.origin.1 as i16;
            let maxx: i32 = col as i32 - posx as i32;
            let maxy: i32 = row as i32 - posy as i32;
            if maxx <= 0 || maxy <= 0 {
                continue;
            }
            let mut y = if posy<0 {1} else {posy + 1};
            let x = if posx<0 {1} else {posx + 1};
            clear_attr!();
            for i in (if posy<0 {-posy as usize} else {0})..(layer.ascii.len().min(row as usize)) {
                move_to!(x, y);
                let out = truncate_visible(layer.ascii[i], maxx as u32, if posx<0 {-posx as u32} else {0});
                print!("{}",out);
                y += 1;
            }
        }
        // Draw password input field
        let posx = calc_pos(&INPUT_LEFT, col);
        let maxx = calc_pos(&INPUT_RIGHT, col);
        let posy = calc_pos(&INPUT_POS, row) - if INPUT_POS.flip {3} else {0};

        let length = maxx-posx-2;

        // If password input field is too small, don't draw it.
        if length<1 || row-(posy as u16)<4 || posy<0 {
            out_password = read_password(0);
        } else {
            let length = length as usize;
            let top_and_bottom: String = std::iter::repeat('-').take(length).collect(); 
            let middle: String = std::iter::repeat(' ').take(length).collect();

            clear_attr!();

            move_to!(posx,posy);
            print!(",");
            print!("{}",&top_and_bottom);
            print!(",");

            move_to!(posx,posy+1);
            print!("|");
            print!("{}",middle);
            print!("|");

            move_to!(posx,posy+2);
            print!("'");
            print!("{}",top_and_bottom);
            print!("'");

            move_to!(posx+1,posy+1);

            out_password = read_password(length);
        }
    },
    Err(_) => {
        //TODO: make configurable
        print!("Unable to get terminal size. Normal password input mode\nPassword: ");
        out_password = read_password(20);
    }
    }
    out_password
}

fn resolve_device_path_from_uuid(uuid: &str) -> std::io::Result<String> {
    let by_uuid_path = Path::new("/dev/disk/by-uuid").join(uuid);

    // 심볼릭 링크 따라가기 (상대 경로일 수 있음)
    let target_path = fs::read_link(&by_uuid_path)?;

    // /dev/disk/by-uuid에서 상대 경로를 따라가면 최종적으로 /dev/sdX 와 같은 절대 경로가 됨
    let absolute_path = by_uuid_path.parent().unwrap().join(target_path).canonicalize()?;

    Ok(absolute_path.display().to_string())
}

fn main() {
    if DEBUG {
        show_input_screen();
        return;
    }
    
    if Path::new("/dev/mapper").join(CRYPTNAME).exists() {
        println!("encrypttui: Device {} already exists, not doing any crypt setup",CRYPTNAME);
        return;
    }
    for _ in 1..TRY_TIMES {
        match resolve_device_path_from_uuid(CRYPTDEVICE_UUID) {
            Ok(path) => {
                loop {
                    // `cryptsetup open ${path} ${CRYPTNAME} --key-file -`
                    let mut args: Vec<String> = Vec::new();
                    args.push("open".to_string());
                    args.push(path.clone());
                    args.push(CRYPTNAME.to_string());
                    args.push("--key-file".to_string());
                    args.push("-".to_string());

                    let mut cryptsetup = Command::new("cryptsetup")
                        .args(&args)
                        .stdin(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .expect("Failed to start cryptsetup! Maybe you need to install cryptsetup");

                    if let Some(mut stdin) = cryptsetup.stdin.take() {
                        stdin.write_all(show_input_screen().as_bytes())
                            .expect("Faileds to write password to cryptsetup stdio!");
                    }
                    let exit_code: i32 = cryptsetup.wait().expect("Failed to wait on cryptsetup!").code().unwrap_or(-1);
                    println!("{}",exit_code);
                    if exit_code == 0 {
                        break;
                    }
                }
                clear_console!();
                return;
            },
            Err(_) => {
                thread::sleep(time::Duration::from_millis(TRY_INTERVAL));
            }
        }
    }
    
}
