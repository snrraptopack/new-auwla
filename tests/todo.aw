

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

match school.get("One"){
    some(s) if s.level > 10 => print(s.name),
    none => print("no name")
}


