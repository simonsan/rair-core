/*
 * core.rs: Linking all rair parts together into 1 module.
 * Copyright (C) 2019  Oddcoder
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use app_dirs::*;
use commands::{Commands, MRc};
use helper::*;
use io::*;
use lineformatter::LineFormatter;
use loc::*;
use rio::*;
use rustyline::{CompletionType, Config, EditMode, Editor, OutputStreamType};
use std::cell::RefCell;
use std::io;
use std::io::Write;
use std::mem;
use std::path::PathBuf;
use std::rc::Rc;
use utils::register_utils;
use writer::Writer;
use yansi::Paint;
pub struct Core {
    pub stdout: Writer,
    pub stderr: Writer,
    pub mode: AddrMode,
    pub io: RIO,
    pub rl: Editor<LineFormatter>,
    loc: u64,
    app_info: AppInfo,
    commands: Rc<RefCell<Commands>>,
    pub color_palette: Vec<(u8, u8, u8)>,
}

impl Default for Core {
    fn default() -> Self {
        Core {
            mode: AddrMode::Phy,
            stdout: Writer::new_write(Box::new(io::stdout())),
            stderr: Writer::new_write(Box::new(io::stderr())),
            io: RIO::new(),
            loc: 0,
            rl: Editor::<LineFormatter>::new(),
            app_info: AppInfo { name: "rair", author: "RairDevs" },
            commands: Default::default(),
            color_palette: Vec::new(),
        }
    }
}
impl Core {
    fn load_commands(&mut self) {
        register_io(self);
        register_loc(self);
        register_utils(self);
    }
    fn init_colors(&mut self) {
        self.color_palette.push((0x58, 0x68, 0x75));
        self.color_palette.push((0xb5, 0x89, 0x00));
        self.color_palette.push((0xcb, 0x4b, 0x16));
        self.color_palette.push((0xdc, 0x32, 0x2f));
        self.color_palette.push((0xd3, 0x36, 0x82));
        self.color_palette.push((0x6c, 0x71, 0xc4));
        self.color_palette.push((0x26, 0x8b, 0xd2));
        self.color_palette.push((0x2a, 0xa1, 0x98));
        self.color_palette.push((0x85, 0x99, 0x00));
    }
    pub fn new() -> Self {
        let mut core: Core = Default::default();
        let config = Config::builder()
            .completion_type(CompletionType::Circular)
            .edit_mode(EditMode::Emacs)
            .output_stream(OutputStreamType::Stdout)
            .build();
        core.rl = Editor::with_config(config);
        core.rl.set_helper(Some(LineFormatter::new(core.commands.clone())));
        core.load_commands();
        drop(core.rl.load_history(&core.hist_file()));
        core.init_colors();
        return core;
    }

    pub fn hist_file(&self) -> PathBuf {
        let mut history = app_dir(AppDataType::UserData, &self.app_info, "/").unwrap();
        history.push("history");
        return history;
    }

    pub fn set_loc(&mut self, loc: u64) {
        self.loc = loc;
    }

    pub fn get_loc(&self) -> u64 {
        self.loc
    }

    pub fn add_command(&mut self, long: &'static str, short: &'static str, funcs: MRc<dyn Cmd>) {
        if !long.is_empty() && !self.commands.borrow_mut().add_command(long, funcs.clone()) {
            let msg = format!("Command {} already existed.", Paint::default(long).bold());
            error_msg(self, "Cannot add this command.", &msg);
        }

        if !short.is_empty() && !self.commands.borrow_mut().add_command(short, funcs) {
            let msg = format!("Command {} already existed.", Paint::default(short).bold());
            error_msg(self, "Cannot add this command.", &msg);
        }
    }
    fn command_not_found(&mut self, command: &str) {
        let msg = format!("Command {} is not found.", Paint::default(command).bold());
        error_msg(self, "Execution failed", &msg);
        let commands = self.commands.borrow();
        let similar = commands.suggest(&command.to_string(), 2);
        let mut s = similar.iter();
        if let Some(suggestion) = s.next() {
            let (r, g, b) = self.color_palette[5];
            write!(self.stderr, "Similar command: {}", Paint::rgb(r, g, b, suggestion)).unwrap();
            for suggestion in s {
                write!(self.stderr, ", {}", Paint::rgb(r, g, b, suggestion)).unwrap();
            }
            writeln!(self.stderr, ".").unwrap();
        }
    }

    pub fn run(&mut self, command: &str, args: &[String]) {
        let cmds = self.commands.clone();
        let cmds_ref = cmds.borrow_mut();
        let cmd = cmds_ref.find(&command.to_string());
        match cmd {
            Some(cmd) => cmd.borrow_mut().run(self, args),
            None => {
                drop(cmds_ref);
                self.command_not_found(command)
            }
        }
    }

    pub fn run_at(&mut self, command: &str, args: &[String], at: u64) {
        let old_loc = mem::replace(&mut self.loc, at);
        self.run(command, args);
        self.loc = old_loc;
    }

    pub fn help(&mut self, command: &str) {
        let cmds = self.commands.clone();
        let cmds_ref = cmds.borrow();
        let cmd = cmds_ref.find(&command.to_string());
        match cmd {
            Some(cmd) => cmd.borrow().help(self),
            None => {
                drop(cmds_ref);
                self.command_not_found(command);
            }
        }
    }
}

#[cfg(test)]
mod test_core {
    use super::*;
    use utils::Quit;
    #[test]
    fn test_loc() {
        let mut core = Core::new();
        core.set_loc(0x500);
        assert_eq!(core.get_loc(), 0x500);
    }
    #[test]
    fn test_add_command() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.add_command("a_non_existing_command", "a", Rc::new(RefCell::new(Quit::new())));
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.add_command("test_command", "s", Rc::new(RefCell::new(Quit::new())));
        assert_eq!(core.stderr.utf8_string().unwrap(), "Error: Cannot add this command.\nCommand s already existed.\n");
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.add_command("seek", "test_stuff", Rc::new(RefCell::new(Quit::new())));
        assert_eq!(core.stderr.utf8_string().unwrap(), "Error: Cannot add this command.\nCommand seek already existed.\n");
    }
    #[test]
    fn test_help() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.help("seeker");
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(core.stderr.utf8_string().unwrap(), "Error: Execution failed\nCommand seeker is not found.\nSimilar command: seek.\n");
    }
    #[test]
    fn test_run_at() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        core.run_at("mep", &[], 0x500);
        assert_eq!(core.stdout.utf8_string().unwrap(), "");
        assert_eq!(
            core.stderr.utf8_string().unwrap(),
            "Error: Execution failed\nCommand mep is not found.\nSimilar command: map, maps, m.\n"
        );
    }
}
