# OS Lab5 Report

## 功能总结

实现了实验指导书中描述的类似银行家算法的死锁检测算法，对 Mutex 和 Semaphore 的死锁分别进行检测。为此，编写了 `DeadlockDetector` 类，并在 `ProcessControlBlockInner` 增加了相应的数据成员。与一般的银行家算法不同，实验框架并没有提供一个用户程序提前申明自己可能使用的最大资源 Need 的系统调用，因此我简单假设 Need 就是当前该线程申请该资源的量，在每次申请资源前，给 Need +1。

## 简答题

> 在我们的多线程实现中，当主线程 (即 0 号线程) 退出时，视为整个进程退出，此时需要结束该进程管理的所有线程并回收其资源。
> - 需要回收的资源有哪些？

需要回收：每个子线程的资源（用户栈、内核栈、TrapContext、控制块数据结构等）、页表、打开的文件、互斥锁、信号量、条件变量等。

> - 其他线程的 TaskControlBlock 可能在哪些位置被引用，分别是否需要回收，为什么？

以下位置持有 `Arc<TaskControlBlock>`：

- 进程自己的线程表 `tasks`。需要主动回收，在 `exit_current_and_run_next` 的最后清空了该表。
- 全局调度器的 `ready_queue`。需要主动回收，因为是全局队列。
- 全局调度器的 `stop_task` 持有主线程的 TCB。需要主动回收，但是延时回收，因为主线程正在自己的内核栈上执行退出流程，不能立刻释放。
- 定时器的 `task`。需要主动回收，因为是全局结构，不回收可能会死后唤醒。
- 所有同步原语（Mutex/Semaphore/Condvar）的 `wait_queue`。不需要主动处理回收，PCB 回收的时候自动就回收了。

> 对比以下两种 `Mutex` 中的实现，二者有什么区别？这些区别可能会导致什么问题？

``` rust
impl Mutex for Mutex1 {
    fn lock(&self) {
        loop {
            let mut mutex_inner = self.inner.exclusive_access();
            if mutex_inner.locked {
                mutex_inner.wait_queue.push_back(current_task().unwrap());
                drop(mutex_inner);
                block_current_and_run_next();
            } else {
                mutex_inner.locked = true;
                break;
            }
        }
    }

    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        mutex_inner.locked = false;
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        }
    }
}

impl Mutex for Mutex2 {
    fn lock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        if mutex_inner.locked {
            mutex_inner.wait_queue.push_back(current_task().unwrap());
            drop(mutex_inner);
            block_current_and_run_next();
        } else {
            mutex_inner.locked = true;
        }
    }

    fn unlock(&self) {
        let mut mutex_inner = self.inner.exclusive_access();
        assert!(mutex_inner.locked);
        if let Some(waking_task) = mutex_inner.wait_queue.pop_front() {
            add_task(waking_task);
        } else {
            mutex_inner.locked = false;
        }
    }
}
```

二者在 `lock` 和 `unlock` 上都存在区别。

首先看 `lock` 的实现：

- `Mutex1` 有一层循环，假如线程获取锁失败而被阻塞，当该线程被再次唤醒时（即从 `block_current_and_run_next()` 返回），仍然需要过一遍循环检查当前能否获取锁；
- `Mutex2` 则没有这一层循环，若一个线程获取锁失败而被阻塞，当它被再次唤醒时，将默认自己已经持有锁，`lock` 函数返回。

这里可能导致问题：`Mutex2` 并不够安全，假如一个线程因为竞争锁失败而沉睡等待，而由于中断等原因被意外唤醒，它将直接从 `lock` 返回并进入临界区，而此时真正的锁持有者也在临界区内，这会造成破坏。

`unlock` 的实现：
- `Mutex1` 始终将 `mutex_inner.locked` 置为 `false`，然后取出一个被阻塞的线程，也即，`unlock` 只是释放锁，然后通知一个等待中的线程去继续竞争，此时该被唤醒的线程与其它同时调用 `lock` 的线程处于平等地位；
- `Mutex2` 在等待队列中有等待的线程时，不将 `mutex_inner.locked` 置为 `false`，而是保持其为 `true` 的同时，唤醒第一个等待的线程，这本质上是将锁的所有权直接移交给等待的线程，该线程天生持有锁，不与其他线程竞争。

这里可能导致问题：`Mutex2` 是严格 FIFO 的，而 `Mutex1` 不是。`Mutex1` 可能导致饥饿问题。

## Honor Code

在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
> 无

此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
> 无

我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。