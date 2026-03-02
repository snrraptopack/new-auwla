// Testing struct destructuring in match and assignment

struct User {
    name: string,
    age: number,
    role: string
}



let alice = User { name: "Alice", age: 30, role: "admin" };
let bob = User { name: "Bob", age: 25, role: "user" };
let charlie = User { name: "Charlie", age: 16, role: "user" };

// Destructure Assignment
let { name, age } = alice;
print("Extracted from Alice: " + name);

// Struct Matching
fn greet(u: User) {
    match u {
        // Shorthand field binding (binding `name` directly)
        { role: "admin", name } => print("Welcome back, Admin {name}"),
        // Conditional structural unwrapping with nested conditional guards
        { role: "user", age } if age < 18 => print("You are not old enough!"),
        // Catch-all structural destructuring fallback
        { name, age } => print("Welcome, {name} ({age})")
    }
}

greet(alice);
greet(bob);
greet(charlie);
