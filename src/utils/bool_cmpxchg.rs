
pub trait BoolCompareExchange {
	fn compare_exchange(&mut self, expected: bool, value: bool) -> bool;
}

impl BoolCompareExchange for bool {
	fn compare_exchange(&mut self, expected: bool, value: bool) -> bool {
		if *self == expected {
			*self = value;
			true
		}
		else {
			false
		}
	}
}
