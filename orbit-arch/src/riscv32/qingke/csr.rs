use qingke::register;

// TODO: Add intsyscr and mtvec

#[allow(non_camel_case_types)]
enum Extentions {
    corecfgr,
    gintenr,
    intsyscr,
    mtvec,
}

pub struct corecfgr(usize);
impl corecfgr {
    #[inline]
    fn read() -> usize {
        register::corecfgr::read()
    }
    #[inline]
    fn write(b: usize) {
        unsafe { register::corecfgr::write(b) }
    }
    #[inline]
    fn set_default() {
        unsafe { register::corecfgr::set_default() }
    }
}

pub struct gintenr(usize);
impl gintenr {
    #[inline]
    pub fn read() -> usize {
        register::gintenr::read()
    }
    #[inline]
    pub fn write(bits: usize) {
        unsafe { register::gintenr::write(bits) }
    }
    #[inline]
    pub fn set_enable() {
        unsafe { register::gintenr::set_enable() }
    }
    #[inline]
    pub fn set_disable() -> usize {
        register::gintenr::set_disable()
    }
}

pub struct ExtendedCSR();
impl ExtendedCSR {
    fn read(reg: Extentions) -> usize {
        match reg {
            Extentions::corecfgr => register::corecfgr::read(),
            Extentions::gintenr => register::gintenr::read(),
            Extentions::intsyscr => todo!(),
            Extentions::mtvec => todo!(),
        }
    }
}
