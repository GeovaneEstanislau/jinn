    .intel_syntax noprefix
    .global irq0_stub
irq0_stub:
    // save general purpose registers
    push rax
    push rcx
    push rdx
    push rbx
    push rbp
    push rsi
    push rdi
    push r8
    push r9
    push r10
    push r11
    push r12
    push r13
    push r14
    push r15

    // call the Rust handler (rdi = pointer to saved regs)
    mov rdi, rsp
    call irq_timer_handler

    // if handler returned non-zero in rax, switch stack to that rsp
    test rax, rax
    jz .no_switch
    mov rsp, rax
.no_switch:

    // restore registers
    pop r15
    pop r14
    pop r13
    pop r12
    pop r11
    pop r10
    pop r9
    pop r8
    pop rdi
    pop rsi
    pop rbp
    pop rbx
    pop rdx
    pop rcx
    pop rax

    // return from interrupt
    iretq
