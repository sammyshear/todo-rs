use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{Read, Seek, Write},
};

use clap::{Arg, ArgAction, Command};

fn main() {
    let item_arg = Arg::new("item")
        .help("Item to add or check off.")
        .action(ArgAction::Set)
        .num_args(1..);

    let xdg_dirs = xdg::BaseDirectories::with_prefix("todo").unwrap();
    let data_path = xdg_dirs
        .place_data_file("todo.txt")
        .expect("could not get data path");
    let mut data_file = OpenOptions::new()
        .write(true)
        .append(false)
        .create(true)
        .read(true)
        .open(data_path)
        .expect("could not create data file");
    let mut buf: Vec<u8> = vec![];
    if data_file.read(&mut buf).unwrap() < 1 {
        let _ = data_file.write_all(b"placeholder:false\n");
    }

    let matches = Command::new("todo")
        .about("cli todo list")
        .version("1.0.0")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .author("Sammy Shear")
        .subcommand(
            Command::new("list")
                .short_flag('l')
                .about("Print the current list."),
        )
        .subcommand(
            Command::new("add")
                .short_flag('a')
                .about("Add to list.")
                .arg(&item_arg),
        )
        .subcommand(
            Command::new("check")
                .short_flag('c')
                .about("Check off an item from the list.")
                .arg(&item_arg),
        )
        .subcommand(
            Command::new("del")
                .short_flag('r')
                .about("delete an item")
                .arg(&item_arg),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("list", _list_matches)) => {
            let list = open_list(data_file);
            list.print();
        }
        Some(("add", add_matches)) => {
            let mut list = open_list(data_file);
            list.list.insert(
                add_matches.get_one::<String>("item").unwrap().to_string(),
                false,
            );
            list.write_to_file();
            list.print();
        }
        Some(("check", check_matches)) => {
            let mut list = open_list(data_file);
            list.list.insert(
                check_matches.get_one::<String>("item").unwrap().to_string(),
                true,
            );
            list.write_to_file();
            list.print();
        }
        Some(("del", del_matches)) => {
            let mut list = open_list(data_file);
            let key = del_matches.get_one::<String>("item").unwrap().to_string();
            list.list.insert(key.clone(), true);
            list.list.remove::<String>(&key);
            list.write_to_file();
            list.print();
        }
        _ => {
            unreachable!()
        }
    }
}

fn open_list(file: File) -> List {
    List::from_file(file)
}

#[derive(Debug)]
struct List {
    file: File,
    list: HashMap<String, bool>,
}

impl List {
    fn from_file(mut file: File) -> List {
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents)
            .expect("Should have been able to read file");
        let mut hash_map = HashMap::<String, bool>::new();

        for s in file_contents.lines() {
            let set = s.split(":").collect::<Vec<&str>>();
            match set[1] {
                "true" => hash_map.insert(set[0].to_string(), true),
                "false" => hash_map.insert(set[0].to_string(), false),
                _ => {
                    unreachable!();
                }
            };
        }

        let list: List = List {
            file,
            list: hash_map,
        };

        list
    }

    fn write_to_file(&mut self) {
        let mut to_write = String::new();
        let _ = &self.file.set_len(0);
        let _ = &self.file.seek(std::io::SeekFrom::Start(0));
        for (item, checked) in &self.list {
            to_write.push_str(&format!("{}:{}\n", item, checked));
        }
        let _ = &self.file.write_all(to_write.as_bytes());
    }

    fn print(&self) {
        let mut i = 0;
        for (item, checked) in &self.list {
            i += 1;
            if *checked {
                println!("{i}. {} ☑", *item);
            } else {
                println!("{i}. {} ☐", *item)
            }
        }
    }
}
