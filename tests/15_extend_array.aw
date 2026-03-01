extend array<T> {
    @external("js", "property", "length")
    fn length(self): number;

    fn last(self): T? {
        if self.length > 0 {
            return some(self[self.length - 1]);
        }
        return none;
    }

    fn first(self): T? {
        if self.length > 0 {
            return some(self[0]);
        }
        return none;
    }
}

let names: string[] = ["Alice", "Bob", "Charlie"];
let last_name = names.last();

match last_name {
    some(name) => print("Last name: {name}"),
    none => print("No names")
}

let numbers: number[] = [1, 2, 3, 4, 5];
let last_num = numbers.last();

match last_num {
    some(n) => print("Last number: {n}"),
    none => print("No numbers")
}
