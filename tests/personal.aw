type Result = T?E;


fn divide(f1:number,f2:number):Result{
    if f2 == 0 { none("can't divide by 0");} else {some(f1/f2);}
}


print(divide(10,0));
print(divide(10,3));
