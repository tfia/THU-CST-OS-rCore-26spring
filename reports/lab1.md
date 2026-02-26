# OS Lab1 Report

## 功能总结

实现了系统调用 `sys_trace`。为了支持该系统调用，修改了 `TaskControlBlock`，在其中增加一个数组成员作为计数器，数组索引为系统调用号，数组值为当前 task 调用的次数。相应地，修改了 `TaskManager`，实现了查询和修改计数器的方法。最后，在内核的 `syscall` 函数中调用修改计数器的方法，统计系统调用的数量。

## 简答题

> 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 三个 bad 测例 (ch2b_bad_*.rs) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

行为如下：

```
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
[kernel] IllegalInstruction in application, kernel killed it.
```

即 `bad_address` 测例出现 PageFault 异常，被内核杀死；后两个测例都是 IllegalInstruction，同样被内核杀死。没有影响内核正常运行。

SBI 版本为 RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0

> 深入理解 trap.S 中两个函数 `__alltraps` 和 `__restore` 的作用，并回答如下问题:
>
> L40：刚进入 `__restore` 时，`sp` 代表了什么值。请指出 `__restore` 的两种使用情景。

`sp` 指向当前任务的内核栈顶，这是在 `__alltraps` 中被设置的。`__restore` 的两种使用情景分别是正常的 trap 处理返回，和任务切换（包括启动第一个应用）。

正常的 trap 处理返回中，`__alltraps` 保存上下文，接着调用 `trap_handler` 处理 trap。`trap_handler` 返回时，会进入 `__restore`，这时候恢复 `__alltraps` 保存的上下文，`sret` 回到 U 态的用户程序。

任务切换时，例如从 Task A 切换到 Task B，`__switch` 函数将 `ra` 和 `sp` 设置为 Task B 的，而 Task B 的 `ra` 指向 `__restore`，`sp` 指向 Task B 自己的内核栈顶。`__switch` 返回时，跳转到 `ra`，开始执行 `__restore`。`__restore` 从 Task B 的内核栈顶读出 Task B 的 `TrapContext`，于是恢复 Task B 的现场，就实现了任务切换。

> L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

这里是在从内核栈顶的 `TrapContext` 中恢复 `sstatus` `sepc` `sscratch` 三个 CSR 寄存器。其中，`sscratch` 恢复之后，保存的是用户栈指针，在 `__restore` 的最后，返回用户程序前被存入了 `sp` 中。

> L50-L56：为何跳过了 `x2` 和 `x4`？

`x2` 是 `sp`，当前指向内核栈，不能急着恢复，因为还要从内核栈里取东西，要最后通过 `sscratch` 换回来。`x4` 是 `tp`，用户程序没有用，无需恢复。

> L60：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

`sp` 是用户栈指针，因为马上就要切换回用户程序了，用户程序需要使用自己的栈。`sscratch` 是内核栈指针，下次 trap 发生时，`__alltraps` 开头的指令会把它交换回 `sp`，这样内核可以用自己的内核栈保存现场。

> `__restore`：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

发生在 `sret` 指令。执行该指令之后硬件根据 `sstatus` 中 SPP 位的值决定之后的特权态。`__restore` 恢复了用户程序的 `sstatus`，其 SPP 被预设为 0，因此回到 U 态。

> L13：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

`sp` 是内核栈指针，`sscratch` 是用户栈指针。意义如前所述。

> 从 U 态进入 S 态是哪一条指令发生的？

用户程序的 `ecall` 指令（对应系统调用）或者异常时发生的。

## Honor Code

在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
> 无

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
> 无

我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。