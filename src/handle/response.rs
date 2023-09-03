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

use std::sync::Arc;
use crate::tasks::request::{TaskRequest, TaskRequestType};
use crate::tasks::response::{TaskStatus, TaskResponse};
use crate::inventory::hosts::Host;
use crate::playbooks::traversal::RunState;
use crate::tasks::fields::Field;
use crate::connection::command::CommandResult;
use crate::playbooks::context::PlaybookContext;
use crate::playbooks::visitor::PlaybookVisitor;
use std::sync::RwLock;

pub struct Response {
    run_state: Arc<RunState>, 
    host: Arc<RwLock<Host>>, 
}

impl Response {

    pub fn new(run_state_handle: Arc<RunState>, host_handle: Arc<RwLock<Host>>) -> Self {
        Self {
            run_state: run_state_handle,
            host: host_handle,
        }
    }

    pub fn get_context(&self) -> Arc<RwLock<PlaybookContext>> {
        return Arc::clone(&self.run_state.context);
    }

    pub fn get_visitor(&self) -> Arc<RwLock<dyn PlaybookVisitor>> {
        return Arc::clone(&self.run_state.visitor);
    }

    pub fn is_failed(&self, _request: &Arc<TaskRequest>,  msg: &String) -> Arc<TaskResponse> {
        return Arc::new(TaskResponse { 
            status: TaskStatus::Failed, 
            changes: Vec::new(), 
            msg: Some(msg.clone()), 
            command_result: Arc::new(None), 
            with: Arc::new(None), 
            and: Arc::new(None)
        });
    }

    pub fn not_supported(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        return self.is_failed(request, &String::from("not supported"));
    }

    pub fn command_failed(&self, _request: &Arc<TaskRequest>, result: &Arc<Option<CommandResult>>) -> Arc<TaskResponse> {
        self.get_visitor().read().expect("read visitor").on_command_failed(&self.get_context(), &Arc::clone(&self.host), &Arc::clone(result));
        return Arc::new(TaskResponse {
            status: TaskStatus::Failed,
            changes: Vec::new(), 
            msg: Some(String::from("command failed")), 
            command_result: Arc::clone(&result), 
            with: Arc::new(None), 
            and: Arc::new(None)
        });
    }

    pub fn command_ok(&self, _request: &Arc<TaskRequest>, result: &Arc<Option<CommandResult>>) -> Arc<TaskResponse> {
        self.get_visitor().read().expect("read visitor").on_command_ok(&self.get_context(), &Arc::clone(&self.host), &Arc::clone(result));
        return Arc::new(TaskResponse {
            status: TaskStatus::IsExecuted,
            changes: Vec::new(), msg: None, command_result: Arc::clone(&result), with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn is_skipped(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Validate, "is_skipped response can only be returned for a validation request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsSkipped, 
            changes: Vec::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn is_matched(&self, request: &Arc<TaskRequest>, ) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "is_matched response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsMatched, 
            changes: Vec::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn is_created(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Create, "is_executed response can only be returned for a creation request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsCreated, 
            changes: Vec::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }
    
    // see also command_ok for shortcuts, as used in the shell module.
    pub fn is_executed(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Execute, "is_executed response can only be returned for a creation request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsExecuted, 
            changes: Vec::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }
    
    pub fn is_removed(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Remove, "is_removed response can only be returned for a remove request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsRemoved, 
            changes: Vec::new(), 
            msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn is_passive(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Passive, "is_passive response can only be returned for a passive request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsPassive, 
            changes: Vec::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }
    
    pub fn is_modified(&self, request: &Arc<TaskRequest>, changes: Vec<Field>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Modify, "is_modified response can only be returned for a modification request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::IsModified, 
            changes: changes, 
            msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn needs_creation(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_creation response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsCreation, 
            changes: Vec::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None), 
        });
    }
    
    pub fn needs_modification(&self, request: &Arc<TaskRequest>, changes: &Vec<Field>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_modification response can only be returned for a query request");
        assert!(!changes.is_empty(), "changes must not be empty");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsModification, 
            changes: changes.clone(), 
            msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None) 
        });
    }
    
    pub fn needs_removal(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_removal response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsRemoval, 
            changes: Vec::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }

    pub fn needs_execution(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_execution response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsExecution, 
            changes: Vec::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None),and: Arc::new(None)
        });
    }
    
    pub fn needs_passive(&self, request: &Arc<TaskRequest>) -> Arc<TaskResponse> {
        assert!(request.request_type == TaskRequestType::Query, "needs_passive response can only be returned for a query request");
        return Arc::new(TaskResponse { 
            status: TaskStatus::NeedsPassive, 
            changes: Vec::new(), msg: None, command_result: Arc::new(None), with: Arc::new(None), and: Arc::new(None)
        });
    }

}