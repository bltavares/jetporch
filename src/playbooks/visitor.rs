
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

// ===================================================================================
// ABOUT: visitor.rs
// these functions may be thought of as callbacks that report what is going on
// with playbook code.  Eventually these themselves may take a vector of additional
// functions, but the plan for now is that they would be overriden in the cli/*.rs
// commands when custom behavior was needed.
// ===================================================================================


use crate::playbooks::context::PlaybookContext;
use crate::tasks::response::{TaskResponse,TaskStatus};
use std::sync::Arc;
use std::sync::RwLock;
use crate::util::terminal::two_column_table;
use crate::inventory::hosts::Host;

use inline_colorization::{color_red,color_blue,color_green,color_cyan,color_reset};

pub trait PlaybookVisitor {

    fn banner(&self) {
        println!("----------------------------------------------------------");
    }

    fn debug(&self, message: String) {
        println!("{color_cyan}| debug | {}{color_reset}", message.clone());
    }

    fn debug_host(&self, host: &Arc<RwLock<Host>>, message: String) {
        println!("{color_cyan}| ..... {} | {}{color_reset}", host.read().unwrap().name, message.clone());
    }

    fn on_playbook_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.playbook_path.lock().unwrap();
        let ctx = context.read().unwrap();
        let path = ctx.playbook_path.as_ref().unwrap();
        self.banner();
        println!("> playbook start: {}", path)
    }

    fn on_play_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.play.lock().unwrap();
        //let play = arc.as_ref().unwrap();
        let play = &context.read().unwrap().play;
        self.banner();
        println!("> play start: {}", play.as_ref().unwrap());
    }
    
    fn on_role_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.role_name.lock().unwrap();
        //let role = arc.as_ref().unwrap();
        let role = &context.read().unwrap().role;
        self.banner();
        println!("> role start: {}", role.as_ref().unwrap());
    }

    fn on_role_stop(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.role_name.lock().unwrap();
        let role = &context.read().unwrap().role;
        self.banner();
        println!("> role stop: {}", role.as_ref().unwrap());
    }

    fn on_play_stop(&self, context: &Arc<RwLock<PlaybookContext>>) {
        let ctx = context.read().unwrap();
        let play_name = ctx.get_play_name();
        self.banner();
        println!("> playbook stop: {}", play_name);
    }

    fn on_exit(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.play.lock().unwrap();
        //let play = arc.as_ref().unwrap();
        

        if self.is_syntax_only() {
            let ctx = context.read().unwrap();
            let play_name = ctx.get_play_name();
            let elements: Vec<(String,String)> = vec![     
                (String::from("Roles"), format!("{}", ctx.get_role_count())),
                (String::from("Tasks"), format!("{}", ctx.get_task_count())),
                (String::from("OK"), String::from("Syntax ok. No configuration attempted.")),
            ];
            two_column_table(&String::from("Play"), &play_name.clone(), &elements);
        } else {
            println!("----------------------------------------------------------");
            println!("> done."); // FIXME: show time here
            println!("");
            show_playbook_summary(context);
        }
    }

    fn on_task_start(&self, context: &Arc<RwLock<PlaybookContext>>) {
        //let arc = context.task.lock().unwrap();
        //let task = arc.as_ref().unwrap();
        //let module = task.get_module();
        let context = context.read().unwrap();
        //let play = context.play;
        let task = context.task.as_ref().unwrap();
        self.banner();
        println!("> begin task: {}", task);
    }

    fn on_batch(&self, batch_num: usize, batch_count: usize, batch_size: usize) {
        self.banner();
        println!("> batch {}/{}, {} hosts", batch_num+1, batch_count, batch_size);
    }

    fn on_task_stop(&self, _context: &Arc<RwLock<PlaybookContext>>) {
        /*
        let context = context.read().unwrap();
        let host = context.host
        let play = context.play;
        let task = context.task;
        println!("@ task complete: {}", task.as_ref().unwrap());
        */
    }

    fn on_host_task_start(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        println!("! host: {} => running", host2.name);
    }

    // FIXME: this pattern of the visitor accessing the context is cleaner than the FSM code that accesses both in sequence, so do
    // more of this below.

    fn on_host_task_ok(&self, context: &Arc<RwLock<PlaybookContext>>, task_response: &Arc<TaskResponse>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        let mut context = context.write().unwrap();
        context.increment_attempted_for_host(&host2.name);
        match &task_response.status {
            TaskStatus::IsCreated  =>  { println!("{color_blue}! host: {} => ok (created){color_reset}",  &host2.name); context.increment_created_for_host(&host2.name);  },
            TaskStatus::IsRemoved  =>  { println!("{color_blue}! host: {} => ok (removed){color_reset}",  &host2.name); context.increment_removed_for_host(&host2.name);  },
            TaskStatus::IsModified =>  { println!("{color_blue}! host: {} => ok (modified){color_reset}", &host2.name); context.increment_modified_for_host(&host2.name); },
            TaskStatus::IsExecuted =>  { println!("{color_blue}! host: {} => ok (executed){color_reset}", &host2.name); context.increment_executed_for_host(&host2.name); },
            TaskStatus::IsPassive  =>  { println!("{color_green}! host: {} => ok (no effect) {color_reset}", &host2.name); context.increment_passive_for_host(&host2.name); }
            TaskStatus::IsMatched  =>  { println!("{color_green}! host: {} => ok (no changes) {color_reset}", &host2.name); } 
            _ => { panic!("on host {}, invalid final task return status, FSM should have rejected: {:?}", host2.name, task_response); }
        }
    }

    fn on_host_task_failed(&self, context: &Arc<RwLock<PlaybookContext>>, task_response: &Arc<TaskResponse>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        println!("{color_red}! host failed: {}{color_reset}", host2.name);
        context.write().unwrap().increment_failed_for_host(&host2.name);
        //println!("> task failed on host: {}", host);
    }

    fn on_host_connect_failed(&self, context: &Arc<RwLock<PlaybookContext>>, host: &Arc<RwLock<Host>>) {
        let host2 = host.read().unwrap();
        context.write().unwrap().increment_failed_for_host(&host2.name);
        println!("{color_red}! connection failed to host: {}{color_reset}", host2.name);
    }

    fn is_syntax_only(&self) -> bool;

    fn is_check_mode(&self) -> bool;

}


pub fn show_playbook_summary(context: &Arc<RwLock<PlaybookContext>>) {
    
    let ctx = context.read().unwrap();
    let play_name = ctx.get_play_name();

    let seen_hosts = ctx.get_hosts_seen_count();
    let role_ct = ctx.get_role_count();
    let task_ct = ctx.get_task_count(); 
    let action_ct = ctx.get_total_attempted_count();
    let action_hosts = ctx.get_hosts_attempted_count();
    let created_ct = ctx.get_total_creation_count();
    let created_hosts = ctx.get_hosts_creation_count();
    let modified_ct = ctx.get_total_modified_count();
    let modified_hosts = ctx.get_hosts_modified_count();
    let removed_ct = ctx.get_total_removal_count();
    let removed_hosts = ctx.get_hosts_removal_count();
    let executed_ct = ctx.get_total_executions_count();
    let executed_hosts = ctx.get_hosts_executions_count();
    let passive_ct = ctx.get_total_passive_count();
    let passive_hosts = ctx.get_hosts_passive_count();
    let adjusted_ct = ctx.get_total_adjusted_count();
    let adjusted_hosts = ctx.get_hosts_adjusted_count();
    let unchanged_hosts = seen_hosts - adjusted_hosts;
    let unchanged_ct = action_ct - adjusted_ct;
    let failed_ct    = ctx.get_total_failed_count();
    let failed_hosts = ctx.get_hosts_failed_count();



    let summary = match failed_hosts {
        0 => match adjusted_hosts {
            0 => String::from("{color_green}(✓) Perfect. All hosts matched policy.{color_reset}"),
            _ => String::from("{color_blue}(✓) Actions were applied.{color_reset}"),
        },
        _ => String::from("{color_red}(X) Failures have occured.{color_reset}"),
    };

    let mode_table = format!("|:-|:-|:-|\n\
                      | Results | Items | Hosts \n\
                      | --- | --- | --- |\n\
                      | Roles | {role_ct} | |\n\
                      | Tasks | {task_ct} | {seen_hosts}|\n\
                      | --- | --- | --- |\n\
                      | Created | {created_ct} | {created_hosts}\n\
                      | Modified | {modified_ct} | {modified_hosts}\n\
                      | Removed | {removed_ct} | {removed_hosts}\n\
                      | Executed | {executed_ct} | {executed_hosts}\n\
                      | Passive | {passive_ct} | {passive_hosts}\n\
                      | --- | --- | ---\n\
                      | Unchanged | {unchanged_ct} | {unchanged_hosts}\n\
                      | Changed | {adjusted_ct} | {adjusted_hosts}\n\
                      | Failed | {failed_ct} | {failed_hosts}\n\
                      |-|-|-");

    crate::util::terminal::markdown_print(&mode_table);
    println!("{}", format!("\n{summary}"));
    println!("");



}