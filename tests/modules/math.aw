// math.aw — a module that exports utility functions and a struct

export struct Vec2 { x: number, y: number }

export fn add(a: number, b: number): number => a + b;

export fn multiply(a: number, b: number): number => a * b;

export fn greet(name: string): string => "Hello, " + name + "!";

export let another = (name:string):string => "your name is {name}";
