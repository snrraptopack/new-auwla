// std/math.aw — Standard library Math namespace

@external("namespace")
type Math {
    @external("js", "method", "floor")
    static fn round_down(x: number): number;

    @external("js", "method", "ceil")
    static fn round_up(x: number): number;

    @external("js", "method", "round")
    static fn round(x: number): number;

    @external("js", "method", "random")
    static fn random(): number;

    @external("js", "method", "abs")
    static fn abs(x: number): number;

    @external("js", "method", "sqrt")
    static fn sqrt(x: number): number;

    @external("js", "method", "pow")
    static fn pow(base: number, exp: number): number;

    @external("js", "method", "min")
    static fn min(a: number, b: number): number;

    @external("js", "method", "max")
    static fn max(a: number, b: number): number;

    @external("js", "property", "PI")
    static fn pi(): number;

    @external("js", "property", "E")
    static fn e(): number;
}
