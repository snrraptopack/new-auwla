extend array<T> {
    @external("js", "property", "length")
    fn len(self): number;

    @external("js", "method", "push")
    fn push_val(self, val: T): void;

    @external("js", "static", "Array", "isArray")
    static fn is_arr(val: number[]): bool;

    fn last(self): T? {
        if self.len() == 0 {
            return none;
        }
        return some(self[self.len() - 1]);
    }

    fn max(self):number{
        var c_max = self[0];
        for i in self{
            if i > c_max{
                c_max = i;
            }
        }
    }
}

fn main() {
    let arr = [1, 2, 3];
    
    // Test instance property mapping
    print("Length: {arr.len()}");
    
    // Test instance method mapping
    arr.push_val(4);
    print("New length: {arr.len()}");
    
    // Test static method mapping
    let check = array::is_arr(arr);
    print("Is array: {check}");
    
    // Test custom extension method calling external one
    match arr.last() {
        some(v) => print("Last: {v}"),
        none => print("Empty"),
    }
}

main();
