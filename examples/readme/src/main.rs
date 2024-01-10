trait F {
    fn f(&self);
}

impl F for i32 {
    fn f(&self) {}
}

fn f(f_arr: &[&dyn F]) {
    for f in f_arr {
        f.f();
    }
}

fn main() {
    for i in 0..10 {
        f(&[&i]);
    }
}
