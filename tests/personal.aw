


let number_array = 1 .. 100
let char_array = 'a' .. 'z'

for i in number_array {
    print("the current value is {i}")
}

fn Divide(first:number, second:number): string?string{
    if second == 0 {
        return none("can't divide by 0")
    }
    return some(first/second)
}
match Divide(10,0){
    some(val) => print("sucess: {val}"),
    none(er) => print("error : {err}")
}

