use neovim_lib::neovim_api::*;
use neovim_lib::IntoVal;
use neovim_lib::*;
use std::collections::*;
use std::hash::*;

macro_rules! log_err {
    ($nvim: expr, $($msg:tt)*) => {{
        let m = format!($($msg)*);
        let _ = $nvim.err_writeln(&m);
        eprintln!("error: {}", m);
    }}
}

fn main() {
    let mut session = Session::new_parent().unwrap();
    session.set_infinity_timeout();

    let mut nvim = Neovim::new(session);
    let receiver = nvim.session.start_event_loop_channel();
    let mut service = Service::new();

    for (event, _values) in receiver {
        // handle msg
        let r = match &event[..] {
            "pop" => service.pop(&mut nvim),
            "push" => service.push(&mut nvim),
            "display" => service.display(&mut nvim),
            m => Err(CallError::GenericError(format!("unknown message: {}", m))),
        };

        // handle erros
        match r {
            Ok(()) => (),
            Err(e) => log_err!(nvim, "{}", e),
        }
    }
}

struct Service {
    stacks: StackMap,
}

struct StackMap(HashMap<WinId, Stack>);

impl StackMap {
    fn new() -> Self {
        StackMap(HashMap::new())
    }

    fn current(&mut self, nvim: &mut Neovim) -> Result<&mut Stack, CallError> {
        let win = nvim.get_current_win()?;
        // let id = WinId(win.get_id(nvim)?);
        let id = WinId(
            nvim.call_function("win_getid", vec![])?
                .as_i64()
                .expect("id was not an i64"),
        );

        if let None = self.0.get(&id) {
            self.0.insert(id, Stack::new(win));
        }
        Ok(self.0.get_mut(&id).unwrap())
    }

    fn clean(&mut self) {
        self.0.retain(|_, x| !x.is_empty());
    }
}

struct Location {
    cursor: (i64, i64),
    buffer: Buffer,
    display: String,
}

#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Copy, Clone)]
struct WinId(i64);

struct Stack {
    win: Window,
    locs: Vec<Location>,
}

impl Stack {
    fn new(win: Window) -> Self {
        Stack {
            win,
            locs: Vec::new(),
        }
    }

    fn is_empty(&self) -> bool {
        self.locs.is_empty()
    }

    fn push(&mut self, l: Location) {
        self.locs.push(l);
    }
}

impl Service {
    fn new() -> Self {
        Service {
            stacks: StackMap::new(),
        }
    }

    fn push(&mut self, nvim: &mut Neovim) -> Result<(), CallError> {
        let stack = self.stacks.current(nvim)?;

        // update
        stack.push(Location {
            cursor: stack.win.get_cursor(nvim)?,
            buffer: nvim.get_current_buf()?,
            display: nvim.get_current_line()?,
        });
        self.display(nvim)?;

        Ok(())
    }

    fn pop(&mut self, nvim: &mut Neovim) -> Result<(), CallError> {
        let stack = self.stacks.current(nvim)?;
        let r = if let Some(loc) = stack.locs.pop() {
            let Location {
                cursor,
                buffer,
                display: _,
            } = loc;
            nvim.set_current_buf(&buffer)?;
            stack.win.set_cursor(nvim, cursor)?;

            self.display(nvim)?;
            Ok(())
        } else {
            Err(CallError::GenericError("empty stack".to_string()))
        };

        self.stacks.clean();

        r
    }

    fn display(&mut self, nvim: &mut Neovim) -> Result<(), CallError> {
        let stack = self.stacks.current(nvim)?;
        let disp = stack
            .locs
            .iter()
            // .map(|l| l.display.trim())
            // .collect::<Vec<&str>>()
            // .join(" |> ");
            .map(|l| &l.display[..])
            .collect::<Vec<&str>>()
            .join("\n");

        // fn quote(s: &str) -> String {
        //     format!("'{}'",s.replace("'", "\\'"))
        // }
        // let disp = quote(&disp);

        // nvim.command(&format!("echo {}", quote(&disp)))?;
        let disp = Utf8String::from(format!("{}\n", disp));
        nvim.call_function("nvim_out_write", vec![Value::String(disp)])?;
        // nvim.call_function("nvim_err_write", vec![Value::String(disp)])?;
        Ok(())

        // nvim.out_write(&disp[..])
    }
}
