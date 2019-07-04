use std::arch::x86_64::_rdtsc;

#[inline]
pub fn get_tsc_cycles() -> u64 {
    unsafe { _rdtsc() as u64 }
}

#[inline]
pub fn get_timer_cycles() -> u64 {
    get_tsc_cycles()
}


// TODO get timer hz

// TODO rte delay us

// TODO rte delay us block

// TODO rte delay us sleep

// TODO rte delay us callback register

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tsc_cycles() {
        let t1 = get_tsc_cycles();
        let t2 = get_tsc_cycles();
        assert!(t1 < t2);
    }
}
