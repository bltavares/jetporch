// Jetporch
// Copyright (C) 2023 - Michael DeHaan <michael@michaeldehaan.net> + contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// long with this program.  If not, see <http://www.gnu.org/licenses/>.

// we don't use any parsing libraries here because they are a bit too automagical
// this may change later.

use std::env;
use std::vec::Vec;
use std::path::PathBuf;
use std::sync::{Arc,RwLock};

pub struct CliParser {
    pub playbook_paths: Arc<RwLock<Vec<PathBuf>>>,
    pub inventory_paths: Arc<RwLock<Vec<PathBuf>>>,
    pub role_paths: Arc<RwLock<Vec<PathBuf>>>,
    pub inventory_set: bool,
    pub playbook_set: bool,
    pub mode: u32,
    pub needs_help: bool,
    pub hosts: Vec<String>,
    pub groups: Vec<String>,
    pub batch_size: Option<usize>,
    pub default_user: String,
    pub default_port: i64,
    pub threads: usize,
    pub verbosity: u32,
    // FIXME: threads and other arguments should be added here.
}

pub const CLI_MODE_UNSET: u32 = 0;
pub const CLI_MODE_SYNTAX: u32 = 1;
pub const CLI_MODE_LOCAL: u32 = 2;
pub const CLI_MODE_CHECK_LOCAL: u32 = 3;
pub const CLI_MODE_SSH: u32 = 4;
pub const CLI_MODE_CHECK_SSH: u32 = 5;
pub const CLI_MODE_SHOW: u32 = 6;

fn is_cli_mode_valid(value: &String) -> bool {
    match cli_mode_from_string(value) {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn cli_mode_from_string(s: &String) -> Result<u32, String> {
    return match s.as_str() {
        "local"       => Ok(CLI_MODE_LOCAL),
        "check-local" => Ok(CLI_MODE_CHECK_LOCAL),
        "ssh"         => Ok(CLI_MODE_SSH),
        "check-ssh"   => Ok(CLI_MODE_CHECK_SSH),
        "syntax"      => Ok(CLI_MODE_SYNTAX),
        "show"        => Ok(CLI_MODE_SHOW),
        _ => Err(format!("invalid mode: {}", s))
    }
}

const ARGUMENT_INVENTORY: &'static str = "--inventory";
const ARGUMENT_INVENTORY_SHORT: &'static str = "-i";
const ARGUMENT_PLAYBOOK: &'static str  = "--playbook";
const ARGUMENT_PLAYBOOK_SHORT: &'static str  = "-p";
const ARGUMENT_ROLES: &'static str  = "--roles";
const ARGUMENT_ROLES_SHORT: &'static str  = "-r";
const ARGUMENT_GROUPS: &'static str = "--groups";
const ARGUMENT_HOSTS: &'static str = "--hosts";
const ARGUMENT_HELP: &'static str = "--help";
const ARGUMENT_PORT: &'static str = "--port";
const ARGUMENT_USER: &'static str = "--user";
const ARGUMENT_USER_SHORT: &'static str = "-u";

const ARGUMENT_THREADS: &'static str = "--threads";
const ARGUMENT_THREADS_SHORT: &'static str = "-t";
const ARGUMENT_BATCH_SIZE: &'static str = "--batch-size";
const ARGUMENT_VERBOSE: &'static str = "-v";
const ARGUMENT_VERBOSER: &'static str = "-vv";
const ARGUMENT_VERBOSEST: &'static str = "-vvv";

fn show_help() {

    let header_table = "|-|:-\n\
                        |jetp | jetporch: the enterprise performance orchestrator |\n\
                        | | (C) Michael DeHaan, 2023\n\
                        | --- | ---\n\
                        | | usage: jetp <MODE> [flags]\n\
                        |-|-";

    println!("");
    crate::util::terminal::markdown_print(&String::from(header_table));
    println!("");

    let mode_table = "|:-|:-|:-\n\
                      | *Category* | *Mode* | *Description*\n\
                      | --- | --- | ---\n\
                      | utility: |\n\
                      | | syntax| scans --playbook files for errors\n\
                      | |\n\
                      | | show | displays inventory, specify --groups or --hosts\n\
                      | |\n\
                      | --- | --- | ---\n\
                      | local machine management: |\n\
                      | | check-local| looks for configuration differences on the local machine\n\
                      | |\n\
                      | | local| manages only the local machine\n\
                      | |\n\
                      | --- | --- | ---\n\
                      | remote machine management: |\n\
                      | | check-ssh | looks for configuration differences over SSH\n\
                      | |\n\
                      | | ssh| manages multiple machines over SSH\n\
                      |-|-";

    crate::util::terminal::markdown_print(&String::from(mode_table));
    println!("");

    let flags_table = "|:-|:-|\n\
                       | *Category* | *Flags* |*Description*\n\
                       | --- | ---\n\
                       | basics:\n\
                       | | -p, --playbook path1:path2| specifies automation content\n\
                       | |\n\
                       | | -r, --roles path1:path2| adds additional role search paths. Also uses $JET_ROLES_PATH\n\
                       | |\n\
                       | --- | ---\n\
                       | SSH specific:\n\
                       | | -i, --inventory path1:path2| (required) specifies which systems to manage\n\
                       | |\n\
                       | | -u, --user username | use this default username instead of $JET_SSH_USER or $USER\n\
                       | |\n\
                       | | --port N | use this default port instead of $JET_SSH_PORT or 22\n\
                       | |\n\
                       | | -t, --threads N| how many parallel threads to use. Alternatively set $JET_THREADS\n\
                       | |\n\
                       | | --batch N| (PENDING FEATURE)\n\
                       | |\n\
                       | --- | ---\n\
                       | scope narrowing:\n\
                       | | --groups group1:group2| limits scope for playbook runs, or used with 'show'\n\
                       | |\n\
                       | | --hosts host1| limits scope for playbook runs, or used with 'show'\n\
                       | |\n\
                       | --- | ---\n\
                       | misc:\n\
                       | | -v -vv -vvv| ever increasing verbosity\n\
                       | |\n\
                       |-|";

    crate::util::terminal::markdown_print(&String::from(flags_table));
    println!("");

}


impl CliParser  {

    pub fn new() -> Self {

        let p = CliParser {
            playbook_paths: Arc::new(RwLock::new(Vec::new())),
            inventory_paths: Arc::new(RwLock::new(Vec::new())),
            role_paths: Arc::new(RwLock::new(Vec::new())),
            needs_help: false,
            mode: CLI_MODE_UNSET,
            hosts: Vec::new(),
            groups: Vec::new(),
            batch_size: None,
            default_user: match env::var("JET_SSH_USER") {
                Ok(x) => {
                    println!("$JET_SSH_USER: {}", x);
                    x
                },
                Err(_) => match env::var("USER") {
                    Ok(y) => y,
                    Err(_) => String::from("root")
                }
            },
            default_port: match env::var("JET_SSH_PORT") {
                Ok(x) => match x.parse::<i64>() {
                    Ok(i)  => {
                        println!("$JET_SSH_PORT: {}", i);
                        i
                    },
                    Err(y) => { println!("environment variable JET_SSH_PORT has an invalid value, ignoring: {}", x); 22 }
                },
                Err(_) => 22
            },
            threads: match env::var("JET_THREADS") {
                Ok(x) => match x.parse::<usize>() {
                        Ok(i)  => i,
                        Err(y) => { println!("environment variable JET_THREADS has an invalid value, ignoring: {}", x); 20 }
                },
                Err(_) => 20
            },
            inventory_set: false,
            playbook_set: false,
            verbosity: 0,
        };
        return p;
    }

    pub fn show_help(&self) {
        show_help();
    }

    pub fn parse(&mut self) -> Result<(), String> {

        let mut arg_count: usize = 0;
        let mut next_is_value = false;

        let args: Vec<String> = env::args().collect();
        'each_argument: for argument in &args {

            let argument_str = argument.as_str();
            arg_count = arg_count + 1;

            match arg_count {
                // the program name doesn't matter
                1 => continue 'each_argument,

                // the second argument is the subcommand name
                2 => {

                    // we should accept --help anywhere, but this is special
                    // handling as with --help we don't need a subcommand
                    if argument == ARGUMENT_HELP {
                        self.needs_help = true;
                        return Ok(())
                    }

                    // if it's not --help, then the second argument is the
                    // required 'mode' parameter
                    let _result = self.store_mode_value(argument)?;
                    continue 'each_argument;
                },

                // for the rest of the arguments we need to pay attention to whether
                // we are reading a flag or a value, which alternate
                _ => {

                    if next_is_value == false {

                        // if we expect a flag...
                        // the --help argument requires special handling as it has no
                        // following value
                        if argument_str == ARGUMENT_HELP {
                            self.needs_help = true;
                            return Ok(())
                        }

                        let result = match argument_str {
                            ARGUMENT_PLAYBOOK          => self.append_playbook_value(&args[arg_count]),
                            ARGUMENT_PLAYBOOK_SHORT    => self.append_playbook_value(&args[arg_count]),
                            ARGUMENT_ROLES             => self.append_roles_value(&args[arg_count]),
                            ARGUMENT_ROLES_SHORT       => self.append_roles_value(&args[arg_count]),
                            ARGUMENT_INVENTORY         => self.append_inventory_value(&args[arg_count]),
                            ARGUMENT_INVENTORY_SHORT   => self.append_inventory_value(&args[arg_count]),
                            ARGUMENT_USER              => self.store_default_user_value(&args[arg_count]),
                            ARGUMENT_USER_SHORT        => self.store_default_user_value(&args[arg_count]),
                            ARGUMENT_GROUPS            => self.store_groups_value(&args[arg_count]),
                            ARGUMENT_HOSTS             => self.store_hosts_value(&args[arg_count]),
                            ARGUMENT_BATCH_SIZE        => self.store_batch_size_value(&args[arg_count]),
                            ARGUMENT_THREADS           => self.store_threads_value(&args[arg_count]),
                            ARGUMENT_THREADS_SHORT     => self.store_threads_value(&args[arg_count]),
                            ARGUMENT_PORT              => self.store_port_value(&args[arg_count]),
                            ARGUMENT_VERBOSE           => self.increase_verbosity(1),
                            ARGUMENT_VERBOSER          => self.increase_verbosity(2),
                            ARGUMENT_VERBOSEST         => self.increase_verbosity(3),
                            _                          => Err(format!("invalid flag: {}", argument_str)),

                        };
                        if result.is_err() { return result; }
                        if argument_str.eq(ARGUMENT_VERBOSE) {
                            // these do not take arguments
                        } else {
                            next_is_value = true;
                        }

                    } else {
                        next_is_value = false;
                        continue 'each_argument;
                    }
                } // end argument numbers 3-N
            }


        }

        // make adjustments based on modes
        match self.mode {
            CLI_MODE_LOCAL       => { self.threads = 1 },
            CLI_MODE_CHECK_LOCAL => { self.threads = 1 },
            CLI_MODE_SYNTAX      => { self.threads = 1 },
            CLI_MODE_SHOW        => { self.threads = 1 },
            CLI_MODE_UNSET       => { self.needs_help = true; },
            _ => {}
        }

        if self.playbook_set {
            self.add_role_paths_from_environment()?;
            self.add_implicit_role_paths()?;
        }

        return self.validate_internal_consistency()
    }

    fn validate_internal_consistency(&mut self) -> Result<(), String> {
        return Ok(());
    }

    fn store_mode_value(&mut self, value: &String) -> Result<(), String> {
        if is_cli_mode_valid(value) {
            self.mode = cli_mode_from_string(value).unwrap();
            return Ok(());
        }
        return Err(format!("jetp mode ({}) is not valid, see --help", value))
     }

    fn append_playbook_value(&mut self, value: &String) -> Result<(), String> {
        self.playbook_set = true;
        match parse_paths(&String::from("-p/--playbook"), value) {
            Ok(paths)  =>  { 
                for p in paths.iter() {
                    self.playbook_paths.write().unwrap().push(p.clone()); 
                }
            },
            Err(err_msg) =>  return Err(format!("--{} {}", ARGUMENT_PLAYBOOK, err_msg)),
        }
        return Ok(());
    }

    fn append_roles_value(&mut self, value: &String) -> Result<(), String> {

        // FIXME: TODO: also load from environment at JET_ROLES_PATH
        match parse_paths(&String::from("-r/--roles"), value) {
            Ok(paths)  =>  { 
                for p in paths.iter() {
                    self.role_paths.write().unwrap().push(p.clone()); 
                }
            },
            Err(err_msg) =>  return Err(format!("--{} {}", ARGUMENT_ROLES, err_msg)),
        }
        return Ok(());
    }

    fn append_inventory_value(&mut self, value: &String) -> Result<(), String> {

        self.inventory_set = true;
        if self.mode == CLI_MODE_LOCAL || self.mode == CLI_MODE_CHECK_LOCAL {
            return Err(format!("--inventory cannot be specified for local modes"));
        }

        match parse_paths(&String::from("-i/--inventory"),value) {
            Ok(paths)  =>  { 
                for p in paths.iter() {
                    self.inventory_paths.write().unwrap().push(p.clone());
                }
            }
            Err(err_msg) =>  return Err(format!("--{} {}", ARGUMENT_INVENTORY, err_msg)),
        }
        return Ok(());
    }

    fn store_groups_value(&mut self, value: &String) -> Result<(), String> {
        match split_string(value) {
            Ok(values)  =>  { self.groups = values; },
            Err(err_msg) =>  return Err(format!("--{} {}", ARGUMENT_GROUPS, err_msg)),
        }
        return Ok(());
    }

    fn store_hosts_value(&mut self, value: &String) -> Result<(), String> {
        match split_string(value) {
            Ok(values)  =>  { self.hosts = values; },
            Err(err_msg) =>  return Err(format!("--{} {}", ARGUMENT_HOSTS, err_msg)),
        }
        return Ok(());
    }

    fn store_default_user_value(&mut self, value: &String) -> Result<(), String> {
        self.default_user = value.clone();
        return Ok(());
    }

    fn store_batch_size_value(&mut self, value: &String) -> Result<(), String> {
        if self.batch_size.is_some() {
            return Err(format!("{} has been specified already", ARGUMENT_BATCH_SIZE));
        }
        match value.parse::<usize>() {
            Ok(n) => { self.batch_size = Some(n); return Ok(()); },
            Err(_e) => { return Err(format!("{}: invalid value",ARGUMENT_BATCH_SIZE)); }
        }
    }

    fn store_threads_value(&mut self, value: &String) -> Result<(), String> {
        match value.parse::<usize>() {
            Ok(n) =>  { self.threads = n; return Ok(()); }
            Err(_e) => { return Err(format!("{}: invalid value", ARGUMENT_THREADS)); }
        }
    }

    fn store_port_value(&mut self, value: &String) -> Result<(), String> {
        match value.parse::<i64>() {
            Ok(n) =>  { self.default_port = n; return Ok(()); }
            Err(_e) => { return Err(format!("{}: invalid value", ARGUMENT_PORT)); }
        }
    }

    fn increase_verbosity(&mut self, amount: u32) -> Result<(), String> {
        self.verbosity = self.verbosity + amount;
        return Ok(())
    }

    fn add_implicit_role_paths(&mut self) -> Result<(), String> {
        let paths = self.playbook_paths.read().unwrap();
        for pb in paths.iter() {
            let dirname = pb.parent().unwrap();
            let mut pathbuf = PathBuf::new();
            pathbuf.push(dirname);
            pathbuf.push("/roles");
            self.role_paths.write().unwrap().push(pathbuf);
        }
        return Ok(());
    }

    fn add_role_paths_from_environment(&mut self) -> Result<(), String> {
        let env_roles_path = env::var("JET_ROLES_PATH");
        if env_roles_path.is_ok() {
            match parse_paths(&String::from("$JET_ROLES_PATH"), &env_roles_path.unwrap()) {
                Ok(paths) => {
                    for p in paths.iter() {
                        println!("role path added from $JET_ROLES_PATH: {:?}", p);
                        self.role_paths.write().unwrap().push(p.to_path_buf());
                    }
                },
                Err(y) => return Err(y)
            };
        }
        return Ok(());
    }

}

fn split_string(value: &String) -> Result<Vec<String>, String> {
    return Ok(value.split(":").map(|x| String::from(x)).collect());
}

// accept paths eliminated by ":" and return a list of paths, provided they exist
fn parse_paths(from: &String, value: &String) -> Result<Vec<PathBuf>, String> {
    let string_paths = value.split(":");
    let mut results = Vec::new();
    for string_path in string_paths {
        let mut path_buf = PathBuf::new();
        path_buf.push(string_path);
        if path_buf.exists() {
            results.push(path_buf)
        } else {
            return Err(format!("path ({}) specified by ({}) does not exist", string_path, from));
        }
    }
    return Ok(results);
}
