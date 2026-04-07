// Test operator overloading

struct Vector2 {
    x: number,
    y: number
}

extend Vector2 {
    operator +(self, other: Vector2): Vector2 {
        return Vector2 {
            x: self.x + other.x,
            y: self.y + other.y
        };
    }

    operator -(self, other: Vector2): Vector2 {
        return Vector2 {
            x: self.x - other.x,
            y: self.y - other.y
        };
    }

    operator *(self, scalar: number): Vector2 {
        return Vector2 {
            x: self.x * scalar,
            y: self.y * scalar
        };
    }

    operator /(self, divisor: number): Vector2 {
        return Vector2 {
            x: self.x / divisor,
            y: self.y / divisor
        };
    }
}

let v1 = Vector2 { x: 10, y: 20 };
let v2 = Vector2 { x: 5, y: 3 };

print("v1 + v2:");
let sum = v1 + v2;
print(sum);


print("v1 - v2:");
let diff = v1 - v2;
print(diff);

print("v1 * 2:");
let scaled = v1 * 2;
print(scaled);

print("v1 / 2:");
let half = v1 / 2;
print(half);

print("(v1 + v2) * 0.5:");
let result = (v1 + v2) * 0.5;
print(result);


let repeated = "Hello " * 3;
print(repeated);



let num_arr = [1 .. 3];

let f = (num_arr.get(0) ?? [0]).get(0) ?? 0;