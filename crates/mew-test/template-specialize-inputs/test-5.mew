use Foo as X;

mod Foo { 
    const bar = 1;
}

mod My_Lib {
  const Foo = 3; // a completely unrelated foo
  mod Demo {
    const one = X::bar;
    @compute fn main() {
        return one;
    }
  }
}

