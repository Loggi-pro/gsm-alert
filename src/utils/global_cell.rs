use core::cell::RefCell;

pub struct GlobalCell<T:Sized>(RefCell<Option<RefCell<T>>>);

unsafe impl<T:Sized> Sync for GlobalCell<T>{}
#[allow(dead_code)]
impl<T:Sized> GlobalCell<T> {
    pub fn get(&self)->& mut T{
        //all variables must to be refs or pointers;
        //we can do here get_mut(), but self is not mutable, so we need to workaround it by ptr;
        let  a =self.0.as_ptr(); //get pointer from refcell
        let b = unsafe{a.as_mut()}.unwrap(); //convert to reference
        let c = b.as_mut().unwrap(); //get reference from inside option
        let d = c.get_mut(); //get reference from inside refcell
        d
    }

    pub const fn new()->Self{
        GlobalCell(RefCell::new(None))
    }
    pub fn set(&self, v:T) {
        *self.0.borrow_mut() = Some(RefCell::new(v));
    }
}
