/*
 * map.rs: commands for mapping/unmapping memory regions and listing mapped regions as well.
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

use core::*;
use helper::*;
use std::io::Write;
use yansi::Paint;

#[derive(Default)]
pub struct Map {}

impl Map {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Cmd for Map {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 3 {
            expect(core, args.len() as u64, 3);
            return;
        }
        let phy;
        let vir;
        let size;
        match str_to_num(&args[0]) {
            Ok(p) => phy = p,
            Err(e) => {
                let name = Paint::default("phy").bold();
                let msg = format!("Failed to parse {}, {}", name, &e.to_string());
                error_msg(core, "Failed to map memory", &msg);
                return;
            }
        }
        match str_to_num(&args[1]) {
            Ok(v) => vir = v,
            Err(e) => {
                let name = Paint::default("vir").bold();
                let msg = format!("Failed to parse {}, {}", name, &e.to_string());
                error_msg(core, "Failed to map memory", &msg);
                return;
            }
        }
        match str_to_num(&args[2]) {
            Ok(s) => size = s,
            Err(e) => {
                let name = Paint::default("size").bold();
                let msg = format!("Failed to parse {}, {}", name, &e.to_string());
                error_msg(core, "Failed to map memory", &msg);
                return;
            }
        }
        if let Err(e) = core.io.map(phy, vir, size) {
            error_msg(core, "Failed to map memory", &e.to_string());
        }
    }
    fn help(&self, core: &mut Core) {
        help(core, &"map", &"", vec![("[phy] [vir] [size]", "Map region from physical address space to virtual address space.")]);
    }
}

#[derive(Default)]
pub struct UnMap {}

impl UnMap {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Cmd for UnMap {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if args.len() != 2 {
            expect(core, args.len() as u64, 2);
            return;
        }
        let vir;
        let size;
        match str_to_num(&args[0]) {
            Ok(v) => vir = v,
            Err(e) => {
                let name = Paint::default("phy").bold();
                let msg = format!("Failed to parse {}, {}", name, &e.to_string());
                error_msg(core, "Failed to unmap memory", &msg);
                return;
            }
        }
        match str_to_num(&args[1]) {
            Ok(s) => size = s,
            Err(e) => {
                let name = Paint::default("vir").bold();
                let msg = format!("Failed to parse {}, {}", name, &e.to_string());
                error_msg(core, "Failed to unmap memory", &msg);
                return;
            }
        }
        if let Err(e) = core.io.unmap(vir, size) {
            error_msg(core, "Failed to unmap memory", &e.to_string());
        }
    }
    fn help(&self, core: &mut Core) {
        help(core, &"unmap", &"um", vec![("[vir] [size]", "Unmap a previosly mapped memory region.")]);
    }
}

#[derive(Default)]
pub struct ListMap {}

impl ListMap {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Cmd for ListMap {
    fn run(&mut self, core: &mut Core, args: &[String]) {
        if !args.is_empty() {
            expect(core, args.len() as u64, 0);
            return;
        }
        let (r, g, b) = core.color_palette[5];
        writeln!(
            core.stdout,
            "{: <20}{: <20}{: <5}",
            Paint::rgb(r, g, b, "Virtual Address"),
            Paint::rgb(r, g, b, "Physical Address"),
            Paint::rgb(r, g, b, "Size")
        )
        .unwrap();
        for map in core.io.map_iter() {
            writeln!(
                core.stdout,
                "{: <20}{: <20}{: <5}",
                format!("0x{:x}", map.vaddr),
                format!("0x{:x}", map.paddr),
                format!("0x{:x}", map.size)
            )
            .unwrap();
        }
    }
    fn help(&self, core: &mut Core) {
        help(core, &"maps", &"", vec![("", "List all memory maps.")]);
    }
}
#[cfg(test)]
mod test_mapping {
    use super::*;
    use writer::Writer;
    use yansi::Paint;
    #[test]
    fn test_map_docs() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let map = Map::new();
        map.help(&mut core);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Command: [map]\n\nUsage:\nmap [phy] [vir] [size]\tMap region from physical address space to virtual address space.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    #[test]
    fn test_unmap_docs() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let unmap = UnMap::new();
        unmap.help(&mut core);
        assert_eq!(
            core.stdout.utf8_string().unwrap(),
            "Commands: [unmap | um]\n\nUsage:\num [vir] [size]\tUnmap a previosly mapped memory region.\n"
        );
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
    #[test]
    fn test_list_map_docs() {
        Paint::disable();
        let mut core = Core::new();
        core.stderr = Writer::new_buf();
        core.stdout = Writer::new_buf();
        let maps = ListMap::new();
        maps.help(&mut core);
        assert_eq!(core.stdout.utf8_string().unwrap(), "Command: [maps]\n\nUsage:\nmaps\tList all memory maps.\n");
        assert_eq!(core.stderr.utf8_string().unwrap(), "");
    }
}
