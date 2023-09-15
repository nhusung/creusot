use crate::{std::cmp::Ordering, *};

#[allow(unused)]
pub trait OrdLogic {
    #[ghost]
    fn cmp_log(self, _: Self) -> Ordering;

    #[ghost]
    #[open]
    fn le_log(self, o: Self) -> bool {
        pearlite! { self.cmp_log(o) != Ordering::Greater }
    }

    #[law]
    #[ensures(x.le_log(y) == (x.cmp_log(y) != Ordering::Greater))]
    fn cmp_le_log(x: Self, y: Self);

    #[ghost]
    #[open]
    fn lt_log(self, o: Self) -> bool {
        pearlite! { self.cmp_log(o) == Ordering::Less }
    }

    #[law]
    #[ensures(x.lt_log(y) == (x.cmp_log(y) == Ordering::Less))]
    fn cmp_lt_log(x: Self, y: Self);

    #[ghost]
    #[open]
    fn ge_log(self, o: Self) -> bool {
        pearlite! { self.cmp_log(o) != Ordering::Less }
    }

    #[law]
    #[ensures(x.ge_log(y) == (x.cmp_log(y) != Ordering::Less))]
    fn cmp_ge_log(x: Self, y: Self);

    #[ghost]
    #[open]
    fn gt_log(self, o: Self) -> bool {
        pearlite! { self.cmp_log(o) == Ordering::Greater }
    }

    #[law]
    #[ensures(x.gt_log(y) == (x.cmp_log(y) == Ordering::Greater))]
    fn cmp_gt_log(x: Self, y: Self);

    #[law]
    #[ensures(x.cmp_log(x) == Ordering::Equal)]
    fn refl(x: Self);

    #[law]
    #[requires(x.cmp_log(y) == o)]
    #[requires(y.cmp_log(z) == o)]
    #[ensures(x.cmp_log(z) == o)]
    fn trans(x: Self, y: Self, z: Self, o: Ordering);

    #[law]
    #[requires(x.cmp_log(y) == Ordering::Less)]
    #[ensures(y.cmp_log(x) == Ordering::Greater)]
    fn antisym1(x: Self, y: Self);

    #[law]
    #[requires(x.cmp_log(y) == Ordering::Greater)]
    #[ensures(y.cmp_log(x) == Ordering::Less)]
    fn antisym2(x: Self, y: Self);

    #[law]
    #[ensures((x == y) == (x.cmp_log(y) == Ordering::Equal))]
    fn eq_cmp(x: Self, y: Self);
}

macro_rules! ord_logic_impl {
    ($t:ty) => {
        impl OrdLogic for $t {
            #[ghost]
            #[open]
            fn cmp_log(self, o: Self) -> Ordering {
                if self < o {
                    Ordering::Less
                } else if self == o {
                    Ordering::Equal
                } else {
                    Ordering::Greater
                }
            }

            #[trusted]
            #[open]
            #[ghost]
            #[creusot::builtins = "int.Int.(<=)"]
            fn le_log(self, _: Self) -> bool {
                true
            }

            #[trusted]
            #[open]
            #[ghost]
            #[creusot::builtins = "int.Int.(<)"]
            fn lt_log(self, _: Self) -> bool {
                true
            }

            #[trusted]
            #[open]
            #[ghost]
            #[creusot::builtins = "int.Int.(>=)"]
            fn ge_log(self, _: Self) -> bool {
                true
            }

            #[trusted]
            #[open]
            #[ghost]
            #[creusot::builtins = "int.Int.(>)"]
            fn gt_log(self, _: Self) -> bool {
                true
            }

            #[ghost]
            #[open(self)]
            fn cmp_le_log(_: Self, _: Self) {
                ()
            }

            #[ghost]
            #[open(self)]
            fn cmp_lt_log(_: Self, _: Self) {
                ()
            }

            #[ghost]
            #[open(self)]
            fn cmp_ge_log(_: Self, _: Self) {
                ()
            }

            #[ghost]
            #[open(self)]
            fn cmp_gt_log(_: Self, _: Self) {
                ()
            }

            #[ghost]
            #[open(self)]
            fn refl(_: Self) {
                ()
            }

            #[ghost]
            #[open(self)]
            fn trans(_: Self, _: Self, _: Self, _: Ordering) {
                ()
            }

            #[ghost]
            #[open(self)]
            fn antisym1(_: Self, _: Self) {
                ()
            }

            #[ghost]
            #[open(self)]
            fn antisym2(_: Self, _: Self) {
                ()
            }

            #[ghost]
            #[open(self)]
            fn eq_cmp(_: Self, _: Self) {
                ()
            }
        }
    };
}

ord_logic_impl!(Int);

ord_logic_impl!(u8);
ord_logic_impl!(u16);
ord_logic_impl!(u32);
ord_logic_impl!(u64);
ord_logic_impl!(u128);
ord_logic_impl!(usize);

ord_logic_impl!(i8);
ord_logic_impl!(i16);
ord_logic_impl!(i32);
ord_logic_impl!(i64);
ord_logic_impl!(i128);
ord_logic_impl!(isize);

impl OrdLogic for bool {
    #[open]
    #[ghost]
    fn cmp_log(self, o : Self) -> Ordering {
        match (self, o) {
            (false, false) => Ordering::Equal,
            (true, true)   => Ordering::Equal,
            (false, true) => Ordering::Less,
            (true, false) => Ordering::Greater,
        }
    }

    #[ghost]
    #[open(self)]
    fn cmp_le_log(_: Self, _: Self) {}

    #[ghost]
    #[open(self)]
    fn cmp_lt_log(_: Self, _: Self) {}

    #[ghost]
    #[open(self)]
    fn cmp_ge_log(_: Self, _: Self) {}

    #[ghost]
    #[open(self)]
    fn cmp_gt_log(_: Self, _: Self) {}

    #[ghost]
    #[open(self)]
    fn refl(_: Self) {}

    #[ghost]
    #[open(self)]
    fn trans(_: Self, _: Self, _: Self, _: Ordering) {}

    #[ghost]
    #[open(self)]
    fn antisym1(_: Self, _: Self) {}

    #[ghost]
    #[open(self)]
    fn antisym2(_: Self, _: Self) {}

    #[ghost]
    #[open(self)]
    fn eq_cmp(_: Self, _: Self) {}
}

impl<A: OrdLogic, B: OrdLogic> OrdLogic for (A, B) {
    #[ghost]
    #[open]
    fn cmp_log(self, o: Self) -> Ordering {
        pearlite! { {
            let r = self.0.cmp_log(o.0);
            if r == Ordering::Equal {
                self.1.cmp_log(o.1)
            } else {
                r
            }
        } }
    }

    #[ghost]
    #[open]
    fn le_log(self, o: Self) -> bool {
        pearlite! { (self.0 == o.0 && self.1 <= o.1) || self.0 <= o.0 }
    }

    #[ghost]
    #[open(self)]
    fn cmp_le_log(_: Self, _: Self) {}

    #[ghost]
    #[open]
    fn lt_log(self, o: Self) -> bool {
        pearlite! { (self.0 == o.0 && self.1 < o.1) || self.0 < o.0 }
    }

    #[ghost]
    #[open(self)]
    fn cmp_lt_log(_: Self, _: Self) {}

    #[ghost]
    #[open]
    fn ge_log(self, o: Self) -> bool {
        pearlite! { (self.0 == o.0 && self.1 >= o.1) || self.0 >= o.0 }
    }

    #[ghost]
    #[open(self)]
    fn cmp_ge_log(_: Self, _: Self) {}

    #[ghost]
    #[open]
    fn gt_log(self, o: Self) -> bool {
        pearlite! { (self.0 == o.0 && self.1 > o.1) || self.0 > o.0 }
    }

    #[ghost]
    #[open(self)]
    fn cmp_gt_log(_: Self, _: Self) {}

    #[ghost]
    #[open(self)]
    fn refl(_: Self) {}

    #[ghost]
    #[open(self)]
    fn trans(_: Self, _: Self, _: Self, _: Ordering) {}

    #[ghost]
    #[open(self)]
    fn antisym1(_: Self, _: Self) {}

    #[ghost]
    #[open(self)]
    fn antisym2(_: Self, _: Self) {}

    #[ghost]
    #[open(self)]
    fn eq_cmp(_: Self, _: Self) {}
}

impl<T: OrdLogic> OrdLogic for Option<T> {
    #[ghost]
    #[open]
    fn cmp_log(self, o: Self) -> Ordering {
        match (self, o) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Less,
            (Some(_), None) => Ordering::Greater,
            (Some(x), Some(y)) => x.cmp_log(y),
        }
    }

    #[law]
    #[open(self)]
    #[ensures(x.le_log(y) == (x.cmp_log(y) != Ordering::Greater))]
    fn cmp_le_log(x: Self, y: Self) {}

    #[law]
    #[open(self)]
    #[ensures(x.lt_log(y) == (x.cmp_log(y) == Ordering::Less))]
    fn cmp_lt_log(x: Self, y: Self) {}

    #[law]
    #[open(self)]
    #[ensures(x.ge_log(y) == (x.cmp_log(y) != Ordering::Less))]
    fn cmp_ge_log(x: Self, y: Self) {}

    #[law]
    #[open(self)]
    #[ensures(x.gt_log(y) == (x.cmp_log(y) == Ordering::Greater))]
    fn cmp_gt_log(x: Self, y: Self) {}

    #[law]
    #[open(self)]
    #[ensures(x.cmp_log(x) == Ordering::Equal)]
    fn refl(x: Self) {}

    #[law]
    #[open(self)]
    #[requires(x.cmp_log(y) == o)]
    #[requires(y.cmp_log(z) == o)]
    #[ensures(x.cmp_log(z) == o)]
    fn trans(x: Self, y: Self, z: Self, o: Ordering) {}

    #[law]
    #[open(self)]
    #[requires(x.cmp_log(y) == Ordering::Less)]
    #[ensures(y.cmp_log(x) == Ordering::Greater)]
    fn antisym1(x: Self, y: Self) {}

    #[law]
    #[open(self)]
    #[requires(x.cmp_log(y) == Ordering::Greater)]
    #[ensures(y.cmp_log(x) == Ordering::Less)]
    fn antisym2(x: Self, y: Self) {}

    #[law]
    #[open(self)]
    #[ensures((x == y) == (x.cmp_log(y) == Ordering::Equal))]
    fn eq_cmp(x: Self, y: Self) {}
}
