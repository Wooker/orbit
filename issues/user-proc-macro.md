# User Proc Macro

Rework proc macro for user applications. Add a unified container type which
accumulates info about an application or a driver.

User can define application or a standalone driver which may be tighed to a
peripheral or a system.

The _orbit_ proc macro automatically implements *Auto traits which define
container characteristics. Every container has a name via _named_ container.
Applications or drivers may be invoked via _invokable_ container. In the case of
a driver, applications have a data struct. Pointer to it is saved in the _data_
container.

If a driver implements a system, the system container saves pointers to
functions of a particular system. However, this scale badly because there should
be only few applications implementing a system. Thus, other containers' size
will be large but most of it will be unused. It's better to keep track of system
implementors in kernel to save application containers' size.
