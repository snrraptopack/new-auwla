

struct School {
    name:string,
    location:string,
    level:number
}



let school : dict<string,School> = {};
let first  = School{name:"One",location:"another",level:10};
let second = School{name:"Second",location:"Taadi",level:0};
school.set(first.name,first);
school.set(second.name,second);

let numbers = [10,30];

let one = numbers.get(0).val_or(0);


print(numbers.get(200) ?? 10);


struct Address {
    city: string,
    country: string,
}

struct User {
    name: string,
    address: Address,
}

let user = User{name:"Ama",address:Address{city:"Tarkwa",country:"Ghana"}};

match user {
    { name, address: { city: "Accra" } } => print("{name} is in Accra"),
    { name, address: { city } }          => print("{name} is in {city}"),
}
