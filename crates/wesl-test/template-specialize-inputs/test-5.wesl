extend My_Lib::Demo;

mod Foo { 
    const bar = 1;
}

mod My_Lib {
  use Foo as X;
  const Foo = 3; // a completely unrelated foo
  mod Demo {
    const one = X::bar;
    @compute fn main() {
        return one;
    }
  }
}

