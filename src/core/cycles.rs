extern "C" {
    #[link_name = "__dpdk_rdtsc"]
    fn dpdk_rdtsc() -> u64;
}

#[inline]
pub fn get_tsc_cycles() -> u64 {
    unsafe { dpdk_rdtsc() }
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

mod tests {
    use super::*;

    #[test]
    fn test_tsc_cycles() {
        let t1 = get_tsc_cycles();
        let t2 = get_tsc_cycles();
        assert!(t1 < t2);
    }
}
