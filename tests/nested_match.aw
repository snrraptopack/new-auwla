struct Address {
    city: string,
    country: string,
}

struct User {
    name: string,
    address: Address,
}

let user = User{
    name:"Ama",
    address:Address{city:"Tarkwa",country:"Ghana"}
};

match user {
    { name, address: { city: "Accra" } } => print("{name} is in Accra"),
    { name, address: { city } }          => print("{name} is in {city}"),
}
