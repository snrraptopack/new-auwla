struct User {
    name: string,
    age: number
}

extend User {
    fn greet(self): string {
        return "Hello, " + self.name;
    }
}

// Test extending a built-in type
extend string {
    fn shout(self): string {
        return self + "!!!";
    }
}

let u: User = User { name: "Amihere", age: 30 };
let msg: string = u.greet();
print(msg);

let s: string = "hello auwla";
print(s.shout());
