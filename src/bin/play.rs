use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Debug)]
struct MyStruct {
    x: Rc<RefCell<Rc<u32>>>,
}

fn main() {
    let s = MyStruct {
        x: Rc::new(RefCell::new(Rc::new(5))),
    };

    {
        let mut v = s.x.borrow_mut();
        *v = Rc::new(6);
    }
    println!("{:?}", s.x.borrow());
}
