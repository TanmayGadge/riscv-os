#[derive(Copy, Clone, PartialEq, Eq)]
pub enum TaskState {
    Empty,
    Ready,
    Running,
    Blocked,    
}

#[repr(C)]
pub struct TaskContext {
    pub ra: usize,
    pub sp: usize,
    pub s: [usize; 12],
}

pub struct TaskControlBlock {
    pub context: TaskContext,
    pub state: TaskState,
    pub stack: [u8; 8192],
}

impl TaskControlBlock {
    pub const fn new() -> Self {
        TaskControlBlock {
            context: TaskContext {
                ra: 0,
                sp: 0,
                s: [0; 12],
            },
            state: TaskState::Empty,
            stack: [0; 8192],
        }
    }
}

const MAX_TASK_NUM: usize = 4;

pub struct TaskManager {
    pub tasks: [TaskControlBlock; MAX_TASK_NUM],
    pub current_task: usize,
    pub task_num: usize,
}

pub static mut TASK_MANAGER: TaskManager = TaskManager {
    tasks: [TaskControlBlock::new(); MAX_TASK_NUM],
    current_task: 0,
    task_num: 0,
};

pub fn add_task(program: fn()) {
    unsafe {
        if TASK_MANAGER.task_num >= MAX_TASK_NUM {
            return; // Cannot add more tasks than the maximum limit
        }
        
        let index = TASK_MANAGER.task_num;
        let task = &mut TASK_MANAGER.tasks[index];
        
        // Stacks grow downwards. We point the Stack Pointer (sp) 
        // to the very end of our allocated stack array.
        let stack_ptr = task.stack.as_ptr() as usize + 8192;
        
        // Initialize the task context
        task.context.ra = program as usize; // Return Address points to the function
        task.context.sp = stack_ptr;        // Stack Pointer points to top of stack
        task.state = TaskState::Ready;      // Mark task as ready to be scheduled
        
        TASK_MANAGER.task_num += 1;
    }
}

use core::arch::global_asm;

global_asm!(
    ".global switch_context",
    "switch_context:",
   
    // Save current task context (a0 points to old TaskContext)
    "sd ra, 0(a0)",
    "sd sp, 8(a0)",
    "sd s0, 16(a0)",
    "sd s1, 24(a0)",
    "sd s2, 32(a0)",
    "sd s3, 40(a0)",
    "sd s4, 48(a0)",
    "sd s5, 56(a0)",
    "sd s6, 64(a0)",
    "sd s7, 72(a0)",
    "sd s8, 80(a0)",
    "sd s9, 88(a0)",
    "sd s10, 96(a0)",
    "sd s11, 104(a0)",

    // Restore next task context (a1 points to new TaskContext)
    "ld ra, 0(a1)",
    "ld sp, 8(a1)",
    "ld s0, 16(a1)",
    "ld s1, 24(a1)",
    "ld s2, 32(a1)",
    "ld s3, 40(a1)",
    "ld s4, 48(a1)",
    "ld s5, 56(a1)",
    "ld s6, 64(a1)",
    "ld s7, 72(a1)",
    "ld s8, 80(a1)",
    "ld s9, 88(a1)",
    "ld s10, 96(a1)",
    "ld s11, 104(a1)",

    // Jump to the restored 'ra'
    "ret"
);

unsafe extern "C" {
    pub fn switch_context(old: *mut TaskContext, new: *const TaskContext);
}

pub fn schedule() {
    unsafe {
        if TASK_MANAGER.task_num <= 1 {
            return; // Nothing to schedule if there is 0 or 1 task
        }

        let current_index = TASK_MANAGER.current_task;
        let mut next_index = (current_index + 1) % TASK_MANAGER.task_num;

        // Simple Round-Robin Scheduler: 
        // Iterate through tasks to find the next one that is 'Ready'
        while TASK_MANAGER.tasks[next_index].state != TaskState::Ready && next_index != current_index {
            next_index = (next_index + 1) % TASK_MANAGER.task_num;
        }

        // If no other task is ready, keep running the current one
        if next_index == current_index {
            return; 
        }

        // 1. Mark current task as Ready (yielding the CPU)
        let current_task = &mut TASK_MANAGER.tasks[current_index];
        if current_task.state == TaskState::Running {
            current_task.state = TaskState::Ready;
        }
        let old_context_ptr = &mut current_task.context as *mut TaskContext;

        // 2. Mark next task as Running
        let next_task = &mut TASK_MANAGER.tasks[next_index];
        next_task.state = TaskState::Running;
        let new_context_ptr = &next_task.context as *const TaskContext;

        // 3. Update the global current task index
        TASK_MANAGER.current_task = next_index;

        // 4. Perform the assembly context switch
        switch_context(old_context_ptr, new_context_ptr);
    }
}

// A helper function a task can call to voluntarily give up the CPU
pub fn yield_task() {
    schedule();
}