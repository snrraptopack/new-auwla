fn main() {
    let arr = [1, 2, 3];

    // This should fail compilation now
    // let val = arr[0];

    // Testing Array.get and val_or
    let val = arr.get(0).val_or(0);
    print("Array get(0).val_or(0): {val}");

    let missing = arr.get(10).val_or(-1);
    print("Array get(10).val_or(-1): {missing}");

    let d = { "a": 1, "b": 2 };
    // This should fail compilation
    // let dv = d["a"];

    // Testing Dict.get and val_or
    let dv = d.get("a").val_or(0);
    print("Dict get(\"a\").val_or(0): {dv}");

    let d_missing = d.get("z").val_or(404);
    print("Dict get(\"z\").val_or(404): {d_missing}");
}

main();
