// std/number.aw — Standard library number extensions

extend number {
    fn abs(self): number {
        if self < 0 {
            return self * -1;
        }
        return self;
    }

    fn double(self): number => self * 2;
    fn square(self): number => self * self;
    fn triple(self): number => self * 3;
    fn by(self, value: number): number => self * value;
    fn add(self, other: number): number => self + other;
    fn sub(self, other: number): number => self - other;
    fn is_even(self): bool {
        let r = self - (Math::round_down(self / 2) * 2);
        return r == 0;
    }
    fn is_odd(self): bool {
        let r = self - (Math::round_down(self / 2) * 2);
        return r != 0;
    }

    fn is_positive(self): bool { return self > 0; }
    fn is_negative(self): bool { return self < 0; }
    fn is_zero(self): bool { return self == 0; }

    fn clamp(self, low: number, high: number): number {
        if self < low { return low; }
        if self > high { return high; }
        return self;
    }
}
