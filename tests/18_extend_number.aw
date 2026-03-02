// Test: extend number — custom methods on primitives used in loops

extend number {
    fn double(self): number => self * 2;
    fn square(self): number => self * self;
    fn triple(self): number => self * 3;
    fn by(self,value:number): number => self * value;
}

// Use in a for loop — call double/square on loop variable
for i in 1 .. 5 {
    let d = i.double();
    let s = i.square();
    print("{i} → double: {d}, square: {s}");
}

// Use in a while loop
var count = 1;
while count < 6 {
    print("{count} tripled = {count.triple()}");
    count = count + 1;
}

// Chain: use one extension inside another
extend number {
    fn double_then_square(self): number => self.double().square();
    fn add(self, other: number): number => self + other;
}

let result = 3.double_then_square();
print("3.double_then_square() = {result}");

// Use in more complex expressions
let nums = [10, 20, 30];
for n in nums {
    let processed = n.double().add(5);
    print("{n}.double().add(5) = {processed}");
}
