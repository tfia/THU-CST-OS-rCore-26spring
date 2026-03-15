# OS Lab3 Report

## 功能总结

我首先迁移了 `sys_get_time` `sys_mmap` `sys_munmap` 系统调用到新的进程框架下。原先在 `TaskManager` 中实现的 `mmap_current` 等方法需要迁移到 `TaskControlBlock` 中，并且修改为每个 `TaskControlBlock` 访问自己的 `MemorySet` 的形式。

接着，我实现了 `sys_spawn` 系统调用。该系统调用直接创建一个新的 `TaskControlBlock`，新进程的 `MemorySet` 通过 `MemorySet::from_elf` 从 ELF 文件创建，并为其准备好 `TrapContext`。最后，将新进程加入 `TaskManager` 的就绪队列中。

之后，我实现了 stride 调度算法。在 `TaskControlBlock` 中添加了 `priority` 和 `pass` 字段，在 `TaskManager` 中实现了 Stride 调度算法，替换原有的 FIFO 调度算法。其中，按照实验手册的建议，我使用暴力扫一遍的方式寻找 pass 最小的进程来调度。

## 简答题

> stride 算法原理非常简单，但是有一个比较大的问题。例如两个 pass = 10 的进程，使用 8bit 无符号整形储存 stride， p1.stride = 255, p2.stride = 250，在 p2 执行一个时间片后，理论上下一次应该 p1 执行。
> - 实际情况是轮到 p1 执行吗？为什么？

不是，p2.stride 发生了溢出，变为 4，比 p1.stride 小，于是下一次仍然是 p2 执行。

> 我们之前要求进程优先级 >= 2 其实就是为了解决这个问题。可以证明，在不考虑溢出的情况下, 在进程优先级全部 >= 2 的情况下，如果严格按照算法执行，那么 STRIDE_MAX – STRIDE_MIN <= BigStride / 2。
> - 为什么？尝试简单说明（不要求严格证明）。

优先级 >= 2，则进程的 pass 值总满足 P.pass <= BigStride / 2。每次选取 stride 值最小的进程运行，每次运行之后 stride 至多增加 BigStride / 2。

初始时满足 STRIDE_MAX – STRIDE_MIN <= BigStride / 2，每轮运行之后，新的 STRIDE_MIN 取值范围为 \[旧 STRIDE_MIN, 旧 STRIDE_MIN + BigStride / 2\]，新的 STRIDE_MAX 取值范围为 \[旧 STRIDE_MAX, 旧 STRIDE_MIN + BigStride / 2\]，因此仍然满足原条件。

> 已知以上结论，考虑溢出的情况下，可以为 Stride 设计特别的比较器，让 `BinaryHeap<Stride>` 的 pop 方法能返回真正最小的 Stride。补全下列代码中的 partial_cmp 函数，假设两个 Stride 永远不会相等。

``` rust
use core::cmp::Ordering;

const BIG_STRIDE: u64 = 1 << 20;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.0 < other.0 {
            if other.0 - self.0 < BIG_STRIDE / 2 {
                Some(Ordering::Less)
            } else {
                Some(Ordering::Greater)
            }
        } else {
            if self.0 - other.0 < BIG_STRIDE / 2 {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Less)
            }
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```


## Honor Code

在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
> 无

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
> 无

我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。