// Test: inline @external with Optional wrapping

extend string {
    @external("js", "method", "charAt")
    fn char_at(self, index: number): string;

    @external("js", "method", "indexOf")
    fn index_of(self, search: string): number;
}

// Non-optional external — just inline
let greeting = "Hello World";
let ch = greeting.char_at(0);
print("First char: {ch}");

let pos = greeting.index_of("World");
print("Position of World: {pos}");

// Test extend with @external property (no Optional)
extend string {
    @external("js", "property", "length")
    fn len(self): number;
}

print("Length of greeting: {greeting.len()}");

// Mix external and pure Auwla
extend string {
    fn shout(self): string => self + "!!!";
    fn whisper(self): string => self + "...";
}

print(greeting.shout());
print(greeting.whisper());

// Use external inside pure Auwla method
extend string {
    fn first_n(self, n: number): string {
        var result = "";
        for i in 0 ..< n {
            result = result + self.char_at(i);
        }
        return result;
    }
}

print("First 5: {greeting.first_n(5)}");
