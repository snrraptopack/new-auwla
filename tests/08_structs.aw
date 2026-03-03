// Tests for user-defined structs

// 1. Definition and Initialization
struct User {
    name: string,
    age: number,
    is_active: bool
}

let alice = User {
    name: "Alice",
    age: 30,
    is_active: true
};

let name = (1 .. 199).len();

print(alice.name);
print(alice.age);
print(alice.is_active);

// mutable updating
var bob = User { name: "Bob", age: 25, is_active: false };
bob.age = 26;
print(bob.age);

// Nested Structs
struct Account {
    user: User,
    balance: number
}


let acc = Account {
    user: alice,
    balance: 1000.50
};



print(acc.user.name);
print(acc.balance);

// 2. Struct Property Validation 
// let user_missing_field_is_an_error = User {
//     name: "Charlie",
//     age: 40
// }; // ERROR EXPECTED: missing field 'is_active'

// let user_wrong_type_is_an_error = User {
//     name: "Dave",
//     age: "forty",
//     is_active: true
// }; // ERROR EXPECTED: age expects 'number', got 'string'

// 3. Duplicate Variable declarations
// let x = 10;
// let x = 20; // ERROR EXPECTED: 'x' already defined in this scope
