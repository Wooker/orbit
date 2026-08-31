# Implement WFI instead of busy loop

At the moment, the kernel has a method that implemens busy loop, called wait. It
was introduced to trigger interrupts while looping in user mode.

However, battery-powered devices cannot rely on this approach. There are
*wfi* or *wfe* instructions which allow the same functionality but need to be
researched.
