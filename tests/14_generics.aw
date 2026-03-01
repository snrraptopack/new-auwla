fn identity<T>(x: T) : T {
    return x;
}



type StringList = string[];

fn main() {
    let x: number = identity(42);
    let y: string = identity("hello");
    let z: string = identity::<string>("forced");

    var again = identity(10);
    again = 10

    let list: StringList = ["a", "b"];
    
    print(x);
    print(y);
    print(z);
    print(list);
}
