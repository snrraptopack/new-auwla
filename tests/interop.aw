@external("js", "namespace")
type Math {
    @external("js", "Math", "floor")
    static fn floor(x: number): number;

    @external("js",  "Math", "random")
    static fn random(): number;

    @external("js", "Math", "PI")
    static fn pi(): number;
}

@external("js", "class")
type Date {
    @external("js", "constructor")
    static fn new(): Date;

    @external("js", "method", "getTime")
    fn get_time(self): number;

    @external("js",  "Date", "now")
    static fn now(): number;
}

fn main() {
    let f = Math::floor(3.9);
    print("Math.floor(3.9) =", f);

    let p = Math::pi();
    print("Math.PI =", p);

    let d = Date::new();
    print("New Date getTime =", d.get_time());

    let n = Date::now();
    print("Date.now() =", n);
}

main();
