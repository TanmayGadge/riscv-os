use core::{
    alloc::{GlobalAlloc, Layout},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::paging::{KERNEL_PAGE_TABLE, PageTable, PageTableEntryFlags};
use spin::{Mutex, MutexGuard};

struct FreePage {
    next: Option<*mut FreePage>,
}
pub struct PhysicalMemoryManager {
    next_free_page: usize,
    free_list: Option<*mut FreePage>,
    limit: usize,
}

impl PhysicalMemoryManager {
    pub const fn new(mut start: usize, limit: usize) -> Self {
        if start % 4096 != 0 {
            start += 4096 - (start % 4096);
        }

        Self {
            next_free_page: start,
            free_list: None,
            limit,
        }
    }

    pub fn alloc_page(&mut self) -> Option<usize> {
        if self.next_free_page + 4096 > self.limit {
            return None;
        }

        if let Some(page_ptr) = self.free_list {
            unsafe {
                let page: &mut FreePage = &mut *page_ptr;
                self.free_list = page.next;

                return Some(page_ptr as usize);
            }
        }

        let address: usize = self.next_free_page;
        // The "Bump" Allocator
        self.next_free_page += 4096;
        Some(address)
    }

    pub fn dealloc_page(&mut self, ptr: usize) {
        let page_ptr: *mut FreePage = ptr as *mut FreePage;
        unsafe {
            (*page_ptr).next = self.free_list;
            self.free_list = Some(page_ptr);
        }
    }
}

struct HeapAllocator {
    pmm: &'static Mutex<PhysicalMemoryManager>,
    next_va: AtomicUsize,
}

impl HeapAllocator {
    pub const fn new(pmm: &'static Mutex<PhysicalMemoryManager>) -> Self {
        Self {
            pmm,
            next_va: AtomicUsize::new(0x4000_0000),
        }
    }
}

unsafe impl GlobalAlloc for HeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size: usize = layout.size();
        let pages_needed: usize = (size + 4095) / 4096;

        let start_va: usize = self
            .next_va
            .fetch_add(pages_needed * 4096, Ordering::SeqCst);

        let mut pmm_lock: MutexGuard<'_, PhysicalMemoryManager> = self.pmm.lock();
        let mut pt_lock: MutexGuard<'_, Option<&'static mut PageTable>> = KERNEL_PAGE_TABLE.lock();

        if let Some(ref mut root_table) = *pt_lock {
            for i in 0..pages_needed {
                let va: usize = start_va + (i * 4096); // implement VMM later

                if let Some(pa) = pmm_lock.alloc_page() {
                    root_table.map(&mut *pmm_lock, va, pa, PageTableEntryFlags::RWX);
                } else {
                    return core::ptr::null_mut();
                }
            }
        }
        unsafe {
            core::arch::asm!("sfence.vma"); // Flush TLB
        }

        start_va as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

// struct VMA{
//     start: usize,
//     end: usize,
//     flags: PageTableEntryFlags,
//     types: usize,

//     next: Option<*mut VMA>
// }
struct VMA {
    start: usize,
    end: usize,
    flags: PageTableEntryFlags,
    types: usize,

    left: Option<*mut VMA>,
    right: Option<*mut VMA>,
    height: isize,
}

impl VMA {
    fn insert(root: Option<*mut VMA>, node: *mut VMA) -> Option<*mut VMA> {
        unsafe {
            if root.is_none() {
                return Some(node);
            }

            let r: *mut VMA = root.unwrap();

            if (*node).start < (*r).start {
                (*r).left = VMA::insert((*r).left, node);
            } else if (*node).start > (*r).start {
                (*r).right = VMA::insert((*r).right, node);
            }

            VMA::update_height(r);

            let balance: isize = VMA::balance_factor(Some(r));

            // left-left insertion case
            if balance > 1 && (*node).start < (*(*r).left.unwrap()).start {
                return Some(VMA::rotate_right(r));
            }

            //right-right insertion case
            if balance < -1 && (*node).start > (*(*r).right.unwrap()).start {
                return Some(VMA::rotate_left(r));
            }

            //left-right insertion case
            if balance > 1 && (*node).start > (*(*r).left.unwrap()).start {
                (*r).left = Some(VMA::rotate_left((*r).left.unwrap()));
                return Some(VMA::rotate_right(r));
            }

            if balance < -1 && (*node).start < (*(*r).right.unwrap()).start {
                (*r).right = Some(VMA::rotate_right((*r).right.unwrap()));
                return Some(VMA::rotate_left(r));
            }

            Some(r)
        }
    }

    fn search(root: Option<*mut VMA>, addr: usize) -> Option<*mut VMA> {
        unsafe {
            let mut current: Option<*mut VMA> = root;

            while let Some(node) = current {
                if addr >= (*node).start && addr < (*node).end {
                    return Some(node);
                }
                if addr > (*node).end {
                    current = (*node).left;
                }
                if addr < (*node).start {
                    current = (*node).left;
                }
            }
        }
        None
    }

    fn delete(root: Option<*mut VMA>, start_addr: usize) -> Option<*mut VMA> {
        unsafe {
            if root == None {
                return None;
            }

            let r: *mut VMA = root.unwrap();
            let mut new_root: Option<*mut VMA> = Some(r);

            if start_addr < (*r).start {

                (*r).left = VMA::delete((*r).left, start_addr);

            } else if start_addr > (*r).start {

                (*r).right = VMA::delete((*r).right, start_addr);

            } else {
                // Node found delete it

                //case 1: no children
                if VMA::is_leaf(r) {
                    new_root = None;
                    // dealloc(r);
                }

                //case 2: one child
                if (*r).left == None && (*r).right != None{
                    new_root = (*r).right;
                  

                    // dealloc(r)
                }
                if (*r).right == None && (*r).left != None{
                    new_root = (*r).left;
                    // dealloc(r)

                }

                //case 3: two children
                if (*r).left != None && (*r).right != None{
                    let successor:*mut VMA = VMA::find_successor(r).unwrap();
                    
                    (*r).start = (*successor).start;
                    (*r).end = (*successor).end;
                    (*r).flags = (*successor).flags.clone();
                    (*r).types = (*successor).types;
    
                    (*r).right = VMA::delete((*r).right, (*successor).start);
                }

            }

            if new_root == None{
                return None;
            }

            (*new_root.unwrap()).height = 1 + VMA::height((*new_root.unwrap()).left).max(VMA::height((*new_root.unwrap()).right));

            let balance = VMA::height((*new_root.unwrap()).left) - VMA::height((*new_root.unwrap()).right);

            if balance > 1 && VMA::balance_factor((*new_root.unwrap()).left) >=0{
                return Some(VMA::rotate_right(new_root.unwrap()));
            }

            if balance > 1 && VMA::balance_factor((*new_root.unwrap()).left) < 0{
                (*new_root.unwrap()).left = Some(VMA::rotate_left((*new_root.unwrap()).left.unwrap()));
                return Some(VMA::rotate_right(new_root.unwrap()));
            }

            if balance < -1 && VMA::balance_factor((*new_root.unwrap()).right) <=0 {
                return Some(VMA::rotate_left(new_root.unwrap()));
            }

            if balance < -1 && VMA::balance_factor((*new_root.unwrap()).right) > 0{
                (*new_root.unwrap()).right = Some(VMA::rotate_right((*new_root.unwrap()).right.unwrap()));
                return Some(VMA::rotate_left(new_root.unwrap()));
            }

            return new_root;
        }
    }

    fn is_leaf(node: *mut VMA) -> bool {
        unsafe {
            if (*node).left == None && (*node).right == None {
                return true;
            }
            false
        }
    }

    fn find_successor(node: *mut VMA) -> Option<*mut VMA>{
        unsafe {
            let mut current:Option<*mut VMA> = (*node).right; 

            if current == None{
                return None;
            }

            while let Some(curr_ptr) = current{
                if let Some(left_ptr) = (*curr_ptr).left{
                    current = Some(left_ptr);
                }else{
                    break;
                }
            }
            return current
        }
    }

    fn rotate_right(node: *mut VMA) -> *mut VMA {
        unsafe {
            let x: *mut VMA = (*node).left.unwrap();
            let t2: Option<*mut VMA> = (*x).right;

            (*x).right = Some(node);
            (*node).right = t2;

            VMA::update_height(x);
            VMA::update_height(node);

            x
        }
    }

    fn rotate_left(node: *mut VMA) -> *mut VMA {
        unsafe {
            let x: *mut VMA = (*node).right.unwrap();
            let t2: Option<*mut VMA> = (*x).left;

            (*x).left = Some(node);
            (*node).left = t2;

            VMA::update_height(x);
            VMA::update_height(node);

            x
        }
    }

    fn height(node: Option<*mut VMA>) -> isize {
        match node {
            Some(n) => unsafe { (*n).height as isize },
            None => 0,
        }
    }

    fn balance_factor(node: Option<*mut VMA>) -> isize {
        if node == None{
            return 0;
        }
        unsafe { VMA::height((*node.unwrap()).left) - VMA::height((*node.unwrap()).right) }
    }

    fn update_height(node: *mut VMA) {
        unsafe {
            let lh: isize = VMA::height((*node).left);
            let rh: isize = VMA::height((*node).right);

            (*node).height = (1 + lh.max(rh)) as isize;
        }
    }

    unsafe fn alloc_vrange(size: usize) {}
    unsafe fn free_vrange(address: usize, size: usize) {}
}

unsafe impl Send for PhysicalMemoryManager {}
unsafe impl Sync for PhysicalMemoryManager {}

unsafe impl Send for HeapAllocator {}
unsafe impl Sync for HeapAllocator {}

pub static PMM: Mutex<PhysicalMemoryManager> = Mutex::new(PhysicalMemoryManager::new(0, 0));

#[global_allocator]
static ALLOCATOR: HeapAllocator = HeapAllocator {
    pmm: &PMM,
    next_va: core::sync::atomic::AtomicUsize::new(0x4000_0000),
};

extern crate alloc;
