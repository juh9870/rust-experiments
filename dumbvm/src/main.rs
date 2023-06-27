mod command {
    pub const DBG: i64 = 0;
    pub const COPY: i64 = 1;
    pub const BAIL: i64 = 2;
    pub const BEGIN: i64 = 3;
    pub const ADD: i64 = 4;
    pub const SUB: i64 = 5;
    pub const MUL: i64 = 6;
    pub const DIV: i64 = 7;
    pub const RESZ: i64 = 8;
}

macro_rules! code_internal {
    (vec![$($items:expr,)*];) => {vec![$($items,)*]};
    (vec![$($items:expr,)*]; $ident:ident $a:tt $b:tt $($rest:tt)*) => {
        code_internal!(vec![$($items,)* command::$ident, $a, $b,];$($rest)*)
    };
}
macro_rules! code {
    ($($rest:tt)*) => {
        code_internal!(vec![];$($rest)*)
    }
}

const COMMAND_WIDTH: i64 = 3;

fn main() {
    let mut code =code! {
        BEGIN 80085 1337
        RESZ 16 17
        ADD 16 1
        SUB 17 2
        DBG 0 0
        BAIL 32 9
    };

    run(&mut code);
}

fn run(tape: &mut Vec<i64>) {
    loop {
        let pointer = tape[0] as usize;

        let result = exec(tape, tape[pointer], tape[pointer + 1], tape[pointer + 2]);
        if result.is_none() {
            return;
        }
        tape[0] += COMMAND_WIDTH;
    }
}

fn exec(tape: &mut Vec<i64>, command: i64, lhs: i64, rhs: i64) -> Option<()> {
    let lhsu = lhs as usize;
    let rhsu = rhs as usize;
    match command {
        command::DBG => {
            println!("{tape:?}")
        }
        command::COPY => {
            tape[lhsu] = tape[rhsu];
        }
        command::BAIL => {
            return None
        }
        command::RESZ => {
            tape.resize(tape[lhsu] as usize, tape[rhsu]);
        }
        command::ADD => {
            tape[lhsu] += tape[rhsu];
        }
        command::SUB => {
            tape[lhsu] -= tape[rhsu];
        }
        command::MUL => {
            tape[lhsu] *= tape[rhsu];
        }
        command::DIV => {
            tape[lhsu] /= tape[rhsu];
        }
        _ => {
            panic!("Bad instruction: {}", command);
        }
    }
    Some(())
}