/* CONTEXT */
#[derive(Clone, Copy)]
pub struct Context {
    pub ra: usize,
    pub sp: usize,
    pub gp: usize,
    pub tp: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    #[cfg(not(target_feature = "e"))]
    pub a6: usize,
    #[cfg(not(target_feature = "e"))]
    pub a7: usize,
    #[cfg(not(target_feature = "e"))]
    pub s2: usize,
    #[cfg(not(target_feature = "e"))]
    pub s3: usize,
    #[cfg(not(target_feature = "e"))]
    pub s4: usize,
    #[cfg(not(target_feature = "e"))]
    pub s5: usize,
    #[cfg(not(target_feature = "e"))]
    pub s6: usize,
    #[cfg(not(target_feature = "e"))]
    pub s7: usize,
    #[cfg(not(target_feature = "e"))]
    pub s8: usize,
    #[cfg(not(target_feature = "e"))]
    pub s9: usize,
    #[cfg(not(target_feature = "e"))]
    pub s10: usize,
    #[cfg(not(target_feature = "e"))]
    pub s11: usize,
    #[cfg(not(target_feature = "e"))]
    pub t3: usize,
    #[cfg(not(target_feature = "e"))]
    pub t4: usize,
    #[cfg(not(target_feature = "e"))]
    pub t5: usize,
    #[cfg(not(target_feature = "e"))]
    pub t6: usize,
    #[cfg(not(target_feature = "e"))]
    pub mepc: usize,
}

impl Context {
    pub const fn new() -> Self {
        Self {
            ra: 0,
            sp: 0,
            gp: 0,
            tp: 0,
            t0: 0,
            t1: 0,
            t2: 0,
            s0: 0,
            s1: 0,
            a0: 0,
            a1: 0,
            a2: 0,
            a3: 0,
            a4: 0,
            a5: 0,
            #[cfg(not(target_feature = "e"))]
            a6: 0,
            #[cfg(not(target_feature = "e"))]
            a7: 0,
            #[cfg(not(target_feature = "e"))]
            s2: 0,
            #[cfg(not(target_feature = "e"))]
            s3: 0,
            #[cfg(not(target_feature = "e"))]
            s4: 0,
            #[cfg(not(target_feature = "e"))]
            s5: 0,
            #[cfg(not(target_feature = "e"))]
            s6: 0,
            #[cfg(not(target_feature = "e"))]
            s7: 0,
            #[cfg(not(target_feature = "e"))]
            s8: 0,
            #[cfg(not(target_feature = "e"))]
            s9: 0,
            #[cfg(not(target_feature = "e"))]
            s10: 0,
            #[cfg(not(target_feature = "e"))]
            s11: 0,
            #[cfg(not(target_feature = "e"))]
            t3: 0,
            #[cfg(not(target_feature = "e"))]
            t4: 0,
            #[cfg(not(target_feature = "e"))]
            t5: 0,
            #[cfg(not(target_feature = "e"))]
            t6: 0,
            #[cfg(not(target_feature = "e"))]
            mepc: 0,
        }
    }
}
