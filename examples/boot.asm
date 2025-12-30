[bits 16]
[org 0x7c00]

KERNEL_OFFSET equ 0x1000    ; العنوان اللي هنحمل فيه الكرنل في الذاكرة

start:
    ; 1. تهيئة السجلات
    xor ax, ax
    mov ds, ax
    mov es, ax
    mov ss, ax
    mov sp, 0x7c00

    ; 2. تحميل الكرنل من القرص
    mov [BOOT_DRIVE], dl    ; BIOS بيخزن رقم القرص في DL، بنحفظه عندنا
    
    mov bx, KERNEL_OFFSET   ; المكان اللي هنحط فيه الكود (ES:BX)
    mov dh, 1               ; عدد القطاعات (Sectors) اللي هنقرأها (زود لو الكرنل كبر)
    mov dl, [BOOT_DRIVE]    ; رقم القرص
    call disk_load

    ; 3. القفز لكود لغتك (الكرنل)
    jmp KERNEL_OFFSET

; --- دالة قراءة القرص ---
disk_load:
    push dx
    mov ah, 0x02            ; وظيفة القراءة من القرص في BIOS
    mov al, dh              ; عدد القطاعات
    mov ch, 0x00            ; Cylinder 0
    mov dh, 0x00            ; Head 0
    mov cl, 0x02            ; ابدأ من القطاع التاني (لأن الأول فيه البوتلودر)
    int 0x13                ; نداء الـ BIOS
    
    jc disk_error           ; لو فيه خطأ (Carry Flag)
    pop dx
    ret

disk_error:
    mov ah, 0x0e
    mov al, 'E'             ; اطبع E لو حصل مشكلة في القرص
    int 0x10
    jmp $

BOOT_DRIVE db 0

; التوقيع
times 510-($-$$) db 0
dw 0xaa55