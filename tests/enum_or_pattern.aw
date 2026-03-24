enum Role {
    Admin,
    Moderator,
    User
}

struct User {
    name: string,
    age: number,
    role: Role
}

let alice = User { name: "Alice", age: 30, role: Role::Admin };
let bob = User { name: "Bob", age: 25, role: Role::User };
let charlie = User { name: "Charlie", age: 28, role: Role::Moderator };

fn greet(u: User) {
    match u {
        // OR pattern in nested position
        { role: Admin | Moderator, name } => print("Welcome back, Staff {name}"),
        { role: User, age } if age < 18 => print("You are not old enough!"),
        { name, age } => print("Welcome, {name} ({age})")
    }
}

greet(alice);
greet(bob);
greet(charlie);
