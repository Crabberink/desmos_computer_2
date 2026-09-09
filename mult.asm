mov ax, 10
mov bx, 12

mult:
	mov cx, 0
	mov dx, 0
	mult_loop:

	cmp cx, bx
	jeq mult_done
	add dx, ax
	add cx, 1
	jmp mult_loop

	mult_done: